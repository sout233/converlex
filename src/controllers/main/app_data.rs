use std::{collections::HashMap, fs, path::Path, sync::Arc};

use rfd::{FileDialog, MessageButtons, MessageDialog, MessageDialogResult, MessageLevel};
use tokio::sync::Mutex;
use uuid::Uuid;
use vizia::prelude::*;

use super::app_event::AppEvent;
use crate::{
    err_msgbox,
    models::{
        convertible_format::ConvertibleFormat,
        media_format::{Audio, MediaFormat, Video},
        task::{Task, TaskStatus, TaskType},
    },
    unwrap_or_msgbox,
    utils::{
        ffmpeg_wrapper::{self, FfmpegEntry, FfmpegTask, ProgressMsg},
        fs::get_file_extension,
        utils::get_output_path,
    },
};
type TaskId = String;

#[derive(Lens, Data, Clone)]
pub struct AppData {
    pub task_ids: Vec<TaskId>,        // 用于显示顺序
    pub tasks: HashMap<TaskId, Task>, // 实际数据
    pub show_config_page: bool,
    pub configuring_taskid: Option<TaskId>,
    pub settings: AppSettings,
    pub show_settings_window: bool,
    pub show_format_selctor_window: bool,
}

impl Model for AppData {
    fn event(&mut self, cx: &mut EventContext, event: &mut Event) {
        event.map(|app_event: &AppEvent, _| match app_event {
            AppEvent::AddTask(name) => {
                let final_name = match name {
                    Some(n) => n.clone(),
                    None => {
                        let video_formats =
                            Video::all().iter().map(|f| f.ext()).collect::<Vec<_>>();
                        let audio_formats =
                            Audio::all().iter().map(|f| f.ext()).collect::<Vec<_>>();
                        match FileDialog::new()
                            .add_filter("video", video_formats.as_slice())
                            .add_filter("audio", audio_formats.as_slice())
                            .add_filter("image", &["jpg", "jpeg", "png", "gif"])
                            .add_filter("All Files", &["*"])
                            .pick_file()
                        {
                            Some(path) => path.to_string_lossy().to_string(),
                            None => return,
                        }
                    }
                };

                let binding = &MediaFormat::new(&get_file_extension(&final_name));
                let arc_formats: Vec<Arc<dyn ConvertibleFormat>> =
                    MediaFormat::get_supported_output_formats(unwrap_or_msgbox!(
                        binding,
                        "不支持的文件格式"
                    ))
                    .into_iter()
                    .map(|boxed| Arc::from(boxed)) // 或 Arc::new(*boxed) if Box is moved
                    .collect();
                let output_format = arc_formats.first().unwrap().as_any();

                let id = Uuid::new_v4().to_string();

                let ffmpeg_entry =
                    unwrap_or_msgbox!(&self.settings.ffmpeg_entry, "未找到ffmpeg，请在设置中配置");
                let task = Task::new(
                    final_name.clone(),
                    get_output_path(&final_name, output_format, false),
                    arc_formats,
                    0,
                    ffmpeg_entry,
                );
                self.tasks.insert(id.clone(), task);

                self.task_ids.push(id);
            }
            AppEvent::RemoveAll => {
                self.task_ids.clear();
                self.tasks.clear();
                self.show_config_page = false;
            }
            AppEvent::ChangeOutputFormat(index, selected_format) => {
                if let Some(task) = self.tasks.get_mut(index) {
                    task.selected_output_format = *selected_format;

                    let format = &*task.supported_output_formats[*selected_format];

                    if task.auto_rename {
                        let new_output_path = get_output_path(&task.input_path, format, false);
                        task.output_path = new_output_path;
                    }
                }
            }
            AppEvent::StartConvert => {
                for task_id in &self.task_ids {
                    let task = &self.tasks[task_id];
                    if task.status != TaskStatus::Queued {
                        continue;
                    }

                    let input_path = &task.input_path;

                    let output_format =
                        task.supported_output_formats[task.selected_output_format].as_any();
                    let mut output_path = get_output_path(input_path, output_format, true);

                    if input_path == &output_path {
                        println!("输入输出路径相同，跳过任务：{}", input_path);
                        rfd::MessageDialog::new()
                            .set_title("Failed")
                            .set_description(format!(
                                "输入输出路径相同，跳过任务：\n\n源文件:\n{}\n输出文件:\n{}",
                                input_path, output_path
                            ))
                            .show();
                        continue;
                    }

                    if Path::new(&output_path).exists() {
                        let overwrite = MessageDialog::new()
                            .set_level(MessageLevel::Warning)
                            .set_title("文件已存在")
                            .set_description(format!(
                                "输出文件已存在，是否覆盖？\n\n源文件:\n{}\n输出文件:\n{}",
                                input_path, output_path
                            ))
                            .set_buttons(MessageButtons::YesNo)
                            .show();

                        match overwrite {
                            MessageDialogResult::Yes => {
                                if let Err(e) = fs::remove_file(&output_path) {
                                    println!("无法删除已存在的文件：{}，错误：{}", output_path, e);
                                    continue;
                                }
                            }
                            MessageDialogResult::No => {
                                output_path = get_output_path(input_path, output_format, false);
                                // *task.output_path = output_path.clone();
                            }
                            _ => {}
                        }
                    }

                    let input_path_clone = input_path.clone();
                    let callback = move |progress: f32| {
                        println!("{:?} 转换进度: {:.2}%", input_path_clone, progress * 100.0);
                    };

                    // tokio::spawn(async move {
                    //     let cb = move |p: f32| {
                    //         // e.g. cx.emit(AppEvent::UpdateProgress(idx, p));
                    //         println!("{} => {:.1}%", input_path_clone.clone(), p * 100.0);
                    //     };
                    //     match crate::utils::utils::convert_media_with_progress(&input_path_clone, &output_path, cb) {
                    //         Ok(_) => println!("Done: {} → {}", input_path_clone, output_path),
                    //         Err(e) => eprintln!("Error: {}", e),
                    //     }
                    // });
                }

                let (tx, rx) = tokio::sync::mpsc::unbounded_channel::<ProgressMsg>();
                let rx = Arc::new(Mutex::new(rx));

                let ffmpeg_entry =
                    unwrap_or_msgbox!(&self.settings.ffmpeg_entry, "未找到ffmpeg，请在设置中配置");

                let tasks: Vec<(String, FfmpegTask)> = self
                    .tasks
                    .iter()
                    .enumerate()
                    .map(|(_, (task_id, task))| {
                        let output_format =
                            Arc::clone(&task.supported_output_formats[task.selected_output_format]);

                        (
                            task_id.clone(),
                            FfmpegTask::new(ffmpeg_entry.clone(), output_format.clone()) //此处报错
                                .input(task.input_path.clone())
                                .output(task.output_path.clone()),
                        )
                    })
                    .collect();

                tokio::spawn(async move {
                    // 🧵 后台并发运行
                    if let Err(e) = ffmpeg_wrapper::run_batch(tasks, tx).await {
                        eprintln!("任务失败：{}", e);
                    }
                });

                let mut event_proxy = cx.get_proxy();

                let rx_clone = rx.clone();
                tokio::spawn(async move {
                    let mut rx = rx_clone.lock().await;
                    while let Some(msg) = rx.recv().await {
                        match msg {
                            ProgressMsg::Progress { task_id, progress } => {
                                let _ = event_proxy
                                    .emit(AppEvent::UpdateProgress(task_id, progress))
                                    .map_err(|e| {
                                        eprintln!("❗ Error emitting PROGRESS event: {}", e);
                                    });
                            }
                            ProgressMsg::Done { task_id } => {
                                let _ = event_proxy
                                    .emit(AppEvent::MarkDone(task_id, true))
                                    .map_err(|e| {
                                        eprintln!("❗ Error emitting COMPLETE event: {}", e);
                                    });
                            }
                            ProgressMsg::Error { task_id, error } => {
                                let _ = event_proxy
                                    .emit(AppEvent::MarkDone(task_id.clone(), false))
                                    .map_err(|e| {
                                        eprintln!("❗ Error emitting ERROR event: {}", e);
                                    });
                                err_msgbox!(format!("Task {task_id} failed\nErr: {error}"))
                            }
                        }
                    }
                });
            }
            AppEvent::ToggleConifg(idx) => {
                self.show_config_page = true;
                self.configuring_taskid = Some(idx.to_string());
            }
            AppEvent::ToggleAutoRename(idx) => {
                if let Some(task) = self.tasks.get_mut(idx) {
                    task.auto_rename = !task.auto_rename;
                    if task.auto_rename {
                        task.output_path = get_output_path(
                            &task.input_path,
                            task.supported_output_formats[task.selected_output_format].as_any(),
                            false,
                        );
                    }
                }
            }
            AppEvent::RemoveTask(_) => todo!(),
            AppEvent::UpdateTask(index, task) => {
                if let Some(existing_task) = self.tasks.get_mut(index) {
                    existing_task.input_path = task.input_path.clone();
                    existing_task.output_path = task.output_path.clone();
                    existing_task.auto_rename = task.auto_rename;
                    existing_task.selected_output_format = task.selected_output_format;
                }
            }
            AppEvent::ConfigWindowClosing => {
                self.show_config_page = false;
                self.configuring_taskid = None;
            }
            AppEvent::UpdateProgress(idx, new_progress) => {
                if let Some(task) = self.tasks.get_mut(idx) {
                    task.progress = *new_progress;
                    task.status = TaskStatus::Running;
                }
            }
            AppEvent::MarkDone(idx, is_sucess) => {
                if let Some(task) = self.tasks.get_mut(idx) {
                    task.progress = 1.0;
                    if *is_sucess {
                        task.status = TaskStatus::Done;
                    } else {
                        task.status = TaskStatus::Failed;
                    }
                }
            }
            AppEvent::UpdateAppSettings(f) => {
                unimplemented!();
                // f(&mut self.settings);
                println!("✅ 更新设置: {:?}", self.settings);
            }
            AppEvent::ToggleSettingsWindow => {
                self.show_settings_window = !self.show_settings_window;
            }
            AppEvent::UpdateFfmpegEntry(app_settings) => {
                self.settings.ffmpeg_entry = app_settings.clone();
            }
            AppEvent::ToggleFormatSelectorWindow(idx) => {
                self.show_format_selctor_window = true;
                self.configuring_taskid = Some(idx.to_string());
            }
            AppEvent::FormatSelectorWindowClosing => {
                self.show_format_selctor_window = false;
            }
        });
    }
}

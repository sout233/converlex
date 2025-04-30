use core::task;
use std::{fs, path::Path, sync::Arc};

use rfd::{FileDialog, MessageButtons, MessageDialog, MessageDialogResult, MessageLevel};
use tokio::sync::{Mutex, mpsc::UnboundedReceiver};
use vizia::prelude::*;

use crate::{
    models::{convertible_format::ConvertibleFormat, media_format::MediaFormat, task::Task},
    utils::{
        ffmpeg_wrapper::{self, FfmpegCommandBuilder, ProgressMsg},
        fs::get_file_extension,
        utils::get_output_path,
    },
};

use super::app_event::AppEvent;

#[derive(Lens, Data, Clone)]
pub struct AppData {
    pub indices: Vec<usize>,
    pub tasks: Vec<Task>,
    pub show_config_window: bool,
    pub configuring_index: Option<usize>,
}

impl Model for AppData {
    fn event(&mut self, cx: &mut EventContext, event: &mut Event) {
        event.map(|app_event: &AppEvent, _| match app_event {
            AppEvent::AddTask(name) => {
                let final_name = match name {
                    Some(n) => n.clone(),
                    None => {
                        match FileDialog::new()
                            .add_filter("video", &["mp4", "mkv", "avi"])
                            .add_filter("audio", &["mp3", "wav", "flac"])
                            .add_filter("image", &["jpg", "jpeg", "png", "gif"])
                            .add_filter("All Files", &["*"])
                            .pick_file()
                        {
                            Some(path) => path.to_string_lossy().to_string(),
                            None => return,
                        }
                    }
                };

                let arc_formats = MediaFormat::get_supported_output_formats(
                    // TODO: 当没有找到格式时，在前端报错
                    &MediaFormat::new(&get_file_extension(&final_name)).unwrap(),
                )
                .into_iter()
                .map(|boxed| Arc::from(boxed)) // 或 Arc::new(*boxed) if Box is moved
                .collect();

                self.tasks.push(Task {
                    input_path: final_name.clone(),
                    output_path: get_output_path(&final_name, &MediaFormat::default(), false),
                    supported_output_formats: arc_formats,
                    done: false,
                    selected_output_format: 0,
                    auto_rename: true,
                    progress: 0.0,
                });
                self.indices.push(self.tasks.len() - 1);
            }
            AppEvent::RemoveAll => {
                self.indices.clear();
                self.tasks.clear();
                self.show_config_window = false;
            }
            AppEvent::ChangeOutputFormat(index, selected_format) => {
                if let Some(task) = self.tasks.get_mut(*index) {
                    task.selected_output_format = *selected_format;

                    let format = &*task.supported_output_formats[*selected_format];
                    let new_output_path = get_output_path(&task.input_path, format, false);

                    task.output_path = new_output_path;
                }
            }
            AppEvent::StartConvert => {
                for index in &self.indices {
                    let task = &self.tasks[*index];
                    if task.done {
                        continue;
                    }

                    let input_path = &task.input_path;

                    let output_format =
                        task.supported_output_formats[task.selected_output_format].as_any();
                    let mut output_path = get_output_path(input_path, output_format, true);

                    if input_path == &output_path {
                        println!("输入输出路径相同，跳过任务：{}", input_path);
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

                let tasks: Vec<(usize, FfmpegCommandBuilder)> = self
                    .tasks
                    .iter()
                    .enumerate()
                    .map(|(idx, task)| {
                        (
                            idx,
                            FfmpegCommandBuilder::new()
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
                                let _ = event_proxy.emit(AppEvent::UpdateProgress(task_id, progress)).map_err(|e| {
                                    eprintln!("❗ Error emitting PROGRESS event: {}", e);
                                });
                            }
                            ProgressMsg::Done { task_id } => {
                                let _ = event_proxy.emit(AppEvent::MarkDone(task_id)).map_err(|e| {
                                    eprintln!("❗ Error emitting COMPLETE event: {}", e);
                                });
                            }
                        }
                    }
                });
            }
            AppEvent::ToggleConifgWindow(idx) => {
                self.show_config_window = !self.show_config_window;
                self.configuring_index = Some(*idx);
            }
            AppEvent::ToggleAutoRename(idx) => {
                if let Some(task) = self.tasks.get_mut(*idx) {
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
                if let Some(existing_task) = self.tasks.get_mut(*index) {
                    existing_task.input_path = task.input_path.clone();
                    existing_task.output_path = task.output_path.clone();
                    existing_task.auto_rename = task.auto_rename;
                    existing_task.selected_output_format = task.selected_output_format;
                }
            }
            AppEvent::ConfigWindowClosing => {
                self.show_config_window = false;
                self.configuring_index = None;
            }
            AppEvent::UpdateProgress(idx, new_progress) => {
                if let Some(task) = self.tasks.get_mut(*idx) {
                    task.progress = *new_progress;
                }
            },
            AppEvent::MarkDone(idx) => {
                if let Some(task) = self.tasks.get_mut(*idx) {
                    task.done = true;
                    task.progress = 1.0;
                }
            },
        });
    }
}

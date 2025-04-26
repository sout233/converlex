use std::{fs, path::Path};

use rfd::{FileDialog, MessageButtons, MessageDialog, MessageDialogResult, MessageLevel};
use vizia::prelude::*;

use crate::{
    app_event::AppEvent,
    media_format::{Audio, MediaFormat, Video},
    task::Task,
    utils::{convert_media, get_output_path},
};

#[derive(Lens, Data, Clone)]
pub struct AppData {
    pub indices: Vec<usize>,
    pub tasks: Vec<Task>,
    pub format_to_list: Vec<MediaFormat>,
    pub selected_format: usize,
    pub show_config_window: bool,
    pub configuring_index: Option<usize>,
}

impl Model for AppData {
    fn event(&mut self, _cx: &mut EventContext, event: &mut Event) {
        event.map(|app_event, _| match app_event {
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

                self.tasks.push(Task {
                    input_path: final_name.clone(),
                    output_path: get_output_path(&final_name, &MediaFormat::default(), false),
                    supported_output_formats: vec![
                        MediaFormat::Audio(Audio::Mp3),
                        MediaFormat::Audio(Audio::Wav),
                        MediaFormat::Audio(Audio::Flac),
                        MediaFormat::Video(Video::Mp4),
                        MediaFormat::Video(Video::Mkv),
                        MediaFormat::Video(Video::Avi),
                    ],
                    done: false,
                    selected_output_format: 0,
                    auto_rename: true,
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

                    let format = &task.supported_output_formats[*selected_format];
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

                    let output_format = &task.supported_output_formats[task.selected_output_format];
                    let mut output_path = get_output_path(input_path, &output_format, true);

                    if input_path == &output_path {
                        println!("输入输出路径相同，跳过任务：{}", input_path);
                        continue;
                    }

                    if Path::new(&output_path).exists() {
                        let overwrite = MessageDialog::new()
                            .set_level(MessageLevel::Warning)
                            .set_title("文件已存在")
                            .set_description(&format!(
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
                                output_path = get_output_path(input_path, &output_format, false)
                            }
                            _ => {}
                        }
                    }

                    match convert_media(input_path, &output_path) {
                        Ok(_) => println!("转换成功：{} -> {}", input_path, output_path),
                        Err(e) => println!("任务转换失败：{}，错误：{}", input_path, e),
                    }
                }
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
                            &task.supported_output_formats[task.selected_output_format],
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
        });
    }
}

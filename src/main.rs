use rfd::{FileDialog, MessageButtons, MessageDialog, MessageDialogResult, MessageLevel};
use std::fmt::Display;
use std::fs;
use std::{path::Path, process::Command};
use vizia::prelude::*;

#[derive(Lens, Data, Clone)]
pub struct AppData {
    indices: Vec<usize>,
    tasks: Vec<Task>,
    format_to_list: Vec<MediaFormat>,
    selected_format: usize,
    show_config_window: bool,
    configuring_index: Option<usize>,
}

#[derive(Data, Clone, Debug, PartialEq)]
pub enum MediaFormat {
    Audio(Audio),
    Video(Video),
}

impl Default for MediaFormat {
    fn default() -> Self {
        MediaFormat::Audio(Audio::Mp3)
    }
}

impl Display for MediaFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MediaFormat::Audio(audio) => write!(f, "{}", audio),
            MediaFormat::Video(video) => write!(f, "{}", video),
        }
    }
}

pub enum AppEvent {
    AddTask(Option<String>),
    RemoveTask(usize),
    ToggleTask(usize),
    ToggleAutoRename(usize),
    ChangeOutputFormat(usize, usize),
    StartConvert,
    RemoveAll,
    ToggleConifgWindow(usize),
}

#[derive(Lens, Data, Clone)]
pub struct Task {
    input_path: String,
    output_path: String,
    // config: ConvertConfig,
    done: bool,
    supported_output_formats: Vec<MediaFormat>,
    selected_output_format: usize,
    auto_rename: bool,
}

#[derive(Lens, Data, Clone)]
pub struct ConvertConfig {}

#[derive(Data, Clone, Debug, PartialEq)]
pub enum Video {
    Mp4,
    Mkv,
    Avi,
}

impl Display for Video {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Video::Mp4 => write!(f, "mp4"),
            Video::Mkv => write!(f, "mkv"),
            Video::Avi => write!(f, "avi"),
        }
    }
}

#[derive(Data, Clone, Debug, PartialEq)]
pub enum Audio {
    Mp3,
    Wav,
    Flac,
}

impl Display for Audio {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Audio::Mp3 => write!(f, "mp3"),
            Audio::Wav => write!(f, "wav"),
            Audio::Flac => write!(f, "flac"),
        }
    }
}

pub struct TaskItemRow {
    task_name: String,
    selected_format: usize,
    format_to_list: Vec<Audio>,
}

impl View for TaskItemRow {}

impl TaskItemRow {
    fn new<L>(cx: &mut Context, task_name: L) -> Handle<Self>
    where
        L: Lens<Target = String>,
    {
        Self {
            task_name: task_name.get(cx),
            selected_format: 0,
            format_to_list: vec![Audio::Mp3, Audio::Wav, Audio::Flac],
        }
        .build(cx, |cx| {
            HStack::new(cx, |cx| {
                // Checkbox::new(cx, task_done).on_toggle(move |cx| {
                //     cx.emit(AppEvent::ToggleTask(cx.index())); // 使用 cx.index() 获取索引
                // });
                Label::new(cx, task_name);
                ComboBox::new(cx, AppData::format_to_list, AppData::selected_format);
            });
        })
    }
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
                    done: false,
                    supported_output_formats: vec![
                        MediaFormat::Audio(Audio::Mp3),
                        MediaFormat::Audio(Audio::Wav),
                        MediaFormat::Audio(Audio::Flac),
                        MediaFormat::Video(Video::Mp4),
                        MediaFormat::Video(Video::Mkv),
                        MediaFormat::Video(Video::Avi),
                    ],
                    selected_output_format: 0,
                    auto_rename: true,
                });
                self.indices.push(self.tasks.len() - 1);
            }
            AppEvent::RemoveAll => {
                self.indices.clear();
                self.tasks.clear();
            }
            AppEvent::ToggleTask(index) => {
                self.tasks[*index].done = !self.tasks[*index].done;
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
        });
    }
}

fn main() -> Result<(), ApplicationError> {
    Application::new(|cx| {
        AppData {
            indices: vec![],
            tasks: vec![],
            format_to_list: vec![
                MediaFormat::Audio(Audio::Mp3),
                MediaFormat::Audio(Audio::Wav),
                MediaFormat::Audio(Audio::Flac),
                MediaFormat::Video(Video::Mp4),
                MediaFormat::Video(Video::Mkv),
                MediaFormat::Video(Video::Avi),
            ],
            selected_format: 0,
            show_config_window: false,
            configuring_index: None,
        }
        .build(cx);

        cx.add_stylesheet(include_style!("src/style.css"))
            .expect("failed to load style");

        VStack::new(cx, |cx| {
            HStack::new(cx, |cx| {
                Button::new(cx, |cx| Label::new(cx, "Add Task"))
                    .on_press(|ex| ex.emit(AppEvent::AddTask(None)));
                Button::new(cx, |cx| Label::new(cx, "Remove All"))
                    .on_press(|ex| ex.emit(AppEvent::RemoveAll));
                Button::new(cx, |cx| Label::new(cx, "Start Convert"))
                    .on_press(|ex| ex.emit(AppEvent::StartConvert));
            })
            .class("menu-btns-row");

            List::new(cx, AppData::indices, |cx, _, idx| {
                Binding::new(cx, idx, |cx, index| {
                    let index = index.get(cx);
                    let item = AppData::tasks.map_ref(move |tasks| &tasks[index]);
                    let input_path = item.then(Task::input_path);
                    let output_path = item.then(Task::output_path);
                    // let supported_output_formats = item.then(Task::supported_output_formats);
                    // let selected_output_format = item.then(Task::selected_output_format);
                    HStack::new(cx, |cx| {
                        Label::new(cx, input_path);
                        Label::new(cx, output_path);
                        // ComboBox::new(cx, supported_output_formats, selected_output_format)
                        //     .on_select(move |cx, selected_format| {
                        //         cx.emit(AppEvent::ChangeOutputFormat(index, selected_format));
                        //     });
                        Button::new(cx, |cx| Label::new(cx, "Config")).on_press(move |cx| {
                            cx.emit(AppEvent::ToggleConifgWindow(index));
                        });
                    });
                    // TaskItemRow::new(cx, AppData::tasks.map_ref(move |tasks| &tasks[index]).get());
                });
            });

            Binding::new(cx, AppData::show_config_window, |cx, is_show| {
                if is_show.get(cx) {
                    Window::popup(cx, true, |cx| {
                        Binding::new(cx, AppData::configuring_index, |cx, index| {
                            let index = index.get(cx);
                            if let Some(index) = index {
                                let item = AppData::tasks.map_ref(move |tasks| &tasks[index]);
                                let input_path = item.then(Task::input_path);
                                let output_path = item.then(Task::output_path);
                                let supported_output_formats =
                                    item.then(Task::supported_output_formats);
                                let selected_output_format =
                                    item.then(Task::selected_output_format);
                                let is_auto_rename = item.then(Task::auto_rename);

                                VStack::new(cx, |cx| {
                                    HStack::new(cx, |cx| {
                                        Label::new(cx, "Input").padding_right(Pixels(10.0));
                                        // Label::new(cx, name);
                                        Textbox::new(cx, input_path).width(Stretch(1.0));
                                    })
                                    .class("config-row");
                                    HStack::new(cx, |cx| {
                                        Label::new(cx, "Output").padding_right(Pixels(10.0));
                                        Textbox::new(cx, output_path)
                                            .width(Stretch(1.0))
                                            .disabled(is_auto_rename);
                                        HStack::new(cx, |cx| {
                                            Label::new(cx, "Auto Rename");
                                            Checkbox::new(cx, is_auto_rename).on_toggle(
                                                move |cx| {
                                                    cx.emit(AppEvent::ToggleAutoRename(index));
                                                },
                                            );
                                        });
                                    })
                                    .class("config-row");
                                    HStack::new(cx, |cx| {
                                        Label::new(cx, "Output Format").width(Stretch(1.0));
                                        ComboBox::new(
                                            cx,
                                            supported_output_formats,
                                            selected_output_format,
                                        )
                                        .alignment(Alignment::Right)
                                        .width(Pixels(100.0))
                                        .on_select(
                                            move |cx, selected_format| {
                                                cx.emit(AppEvent::ChangeOutputFormat(
                                                    index,
                                                    selected_format,
                                                ));
                                            },
                                        );
                                    })
                                    .class("config-row");
                                });
                            }
                        });
                    });
                }
            });
        })
        .on_drop(|ex, data| {
            if let DropData::File(file) = data {
                println!("Dropped File: {:?}", file);
                ex.emit(AppEvent::AddTask(Some(
                    file.to_str().unwrap_or_default().to_string(),
                )));
            }
        })
        .on_hover(|ex| {
            if ex.has_drop_data() {
                ex.emit(WindowEvent::SetCursor(CursorIcon::Copy));
            } else {
                ex.emit(WindowEvent::SetCursor(CursorIcon::Default));
            }
        })
        .class("main-container");
    })
    .title("Converlex")
    .inner_size((1000, 400))
    .anchor(Anchor::Center)
    .parent_anchor(Anchor::Center)
    .anchor_target(AnchorTarget::Monitor)
    .run()
}

fn convert_media(input: &str, output: &str) -> Result<(), String> {
    let ffmpeg_path = "./ffmpeg.exe"; // 确保可执行文件打包进程序目录
    if !Path::new(ffmpeg_path).exists() {
        return Err("找不到 ffmpeg.exe".to_string());
    }

    let status = Command::new(ffmpeg_path)
        .args(["-i", input, output])
        .status()
        .map_err(|e| format!("启动失败: {}", e))?;

    if status.success() {
        Ok(())
    } else {
        Err("ffmpeg 转换失败".into())
    }
}

fn get_output_path(input_path: &str, new_ext: &MediaFormat, overwrite: bool) -> String {
    let path = Path::new(input_path);
    let stem = path.file_stem().unwrap_or_default().to_string_lossy();
    let parent = path.parent().unwrap_or_else(|| Path::new(""));

    let mut output_path = parent.join(format!("{}_converted.{}", stem, new_ext));
    let mut count = 1;

    if !overwrite {
        while output_path.exists() {
            output_path = parent.join(format!("{}_converted_{}.{}", stem, count, new_ext));
            count += 1;
        }
    }

    output_path.to_string_lossy().to_string()
}

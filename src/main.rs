use rfd::{FileDialog, MessageButtons, MessageDialog, MessageDialogResult, MessageLevel};
use std::fmt::Display;
use std::{path::Path, process::Command};
use vizia::prelude::*;

#[derive(Lens, Data, Clone)]
pub struct AppData {
    indices: Vec<usize>,
    tasks: Vec<Task>,
    format_to_list: Vec<Audio>,
    selected_format: usize,
}

pub enum AppEvent {
    AddTask(Option<String>),
    RemoveTask(usize),
    ToggleTask(usize),
    StartConvert,
    RemoveAll,
}

#[derive(Lens, Data, Clone)]
pub struct Task {
    name: Option<String>,
    // config: ConvertConfig,
    done: bool,
}

#[derive(Lens, Data, Clone)]
pub struct ConvertConfig {

}

enum Video {
    Mp4,
    Mkv,
    Avi,
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
            Audio::Mp3 => write!(f, "MP3"),
            Audio::Wav => write!(f, "WAV"),
            Audio::Flac => write!(f, "FLAC"),
        }
    }
}

pub struct TaskItemRow{
    task_name: String,
    selected_format: usize,
    format_to_list: Vec<Audio>,
}

impl View for TaskItemRow {}

impl TaskItemRow {
    fn new<L>(cx: &mut Context, task_name: L) -> Handle<Self> 
    where L: Lens<Target = String> {
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
                    name: Some(final_name),
                    done: false,
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
            AppEvent::StartConvert => {
                let tasks = self.tasks.clone();
                let indices = self.indices.clone();
                for index in indices {
                    let task = &tasks[index];
                    if !task.done {
                        if let Some(input_path) = &task.name {
                            let output_path = get_output_path(input_path, "mp3");

                            if input_path == &output_path {
                                println!("输入输出路径相同，跳过！");
                                continue;
                            }

                            let choice = MessageDialog::new()
                                .set_level(MessageLevel::Info)
                                .set_title("转换确认")
                                .set_description(&format!(
                                    "是否删除源文件？\n\n源文件:\n{}\n输出文件:\n{}",
                                    input_path, output_path
                                ))
                                .set_buttons(MessageButtons::YesNoCancel)
                                .show();

                            match choice {
                                MessageDialogResult::Yes => {
                                    if convert_media(input_path, &output_path).is_ok() {
                                        let _ = std::fs::remove_file(input_path);
                                        println!("转换成功并删除源文件：{}", input_path);
                                    } else {
                                        println!("任务转换失败：{}", input_path);
                                    }
                                }
                                MessageDialogResult::No => {
                                    if convert_media(input_path, &output_path).is_err() {
                                        println!("任务转换失败：{}", input_path);
                                    }
                                }
                                MessageDialogResult::Cancel => {
                                    println!("取消转换：{}", input_path);
                                }
                                MessageDialogResult::Ok => todo!(),
                                MessageDialogResult::Custom(_) => todo!(),
                            }
                        }
                    }
                }
            }
            _ => {}
        });
    }
}

fn main() -> Result<(), ApplicationError> {
    Application::new(|cx| {
        AppData {
            indices: vec![],
            tasks: vec![],
            format_to_list: vec![Audio::Mp3, Audio::Wav, Audio::Flac],
            selected_format: 0,
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
                    let name = item.get(cx).name;
                    // TaskItemRow::new(cx, AppData::tasks.map_ref(move |tasks| &tasks[index]).get());
                });
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
        println!("转换成功：{} -> {}", input, output);
        Ok(())
    } else {
        Err("ffmpeg 转换失败".into())
    }
}

fn get_output_path(input_path: &str, new_ext: &str) -> String {
    let path = Path::new(input_path);
    let stem = path.file_stem().unwrap_or_default().to_string_lossy();
    let parent = path.parent().unwrap_or_else(|| Path::new(""));

    let mut output_path = parent.join(format!("{}_converted.{}", stem, new_ext));
    let mut count = 1;

    while output_path.exists() {
        output_path = parent.join(format!("{}_converted_{}.{}", stem, count, new_ext));
        count += 1;
    }

    output_path.to_string_lossy().to_string()
}

use app_data_derived_lenses::show_config_window;
use rfd::{FileDialog, MessageButtons, MessageDialog, MessageDialogResult, MessageLevel};
use std::fmt::Display;
use std::fs;
use std::{path::Path, process::Command};
use vizia::prelude::*;
use vizia::views::combo_box_derived_lenses::selected;

#[derive(Lens, Data, Clone)]
pub struct AppData {
    indices: Vec<usize>,
    tasks: Vec<Task>,
    format_to_list: Vec<Audio>,
    selected_format: usize,
    show_config_window:bool,
}

pub enum AppEvent {
    AddTask(Option<String>),
    RemoveTask(usize),
    ToggleTask(usize),
    ChangeOutputFormat(usize, usize),
    StartConvert,
    RemoveAll,
    ShowConifgWindow(usize),
}

#[derive(Lens, Data, Clone)]
pub struct Task {
    name: Option<String>,
    // config: ConvertConfig,
    done: bool,
    supported_output_formats: Vec<Audio>,
    selected_output_format: usize,
}

#[derive(Lens, Data, Clone)]
pub struct ConvertConfig {}

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
                    name: Some(final_name),
                    done: false,
                    supported_output_formats: vec![Audio::Mp3, Audio::Wav, Audio::Flac],
                    selected_output_format: 0,
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
                self.tasks[*index].selected_output_format = *selected_format;
            }
            AppEvent::StartConvert => {
                for index in &self.indices {
                    let task = &self.tasks[*index];
                    if task.done {
                        continue;
                    }

                    if let Some(input_path) = &task.name {
                        let output_format =
                            &task.supported_output_formats[task.selected_output_format];
                        let mut output_path = get_output_path(input_path, &output_format.to_string(),true);

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

                            match overwrite{
                                MessageDialogResult::Yes => {
                                    if let Err(e) = fs::remove_file(&output_path) {
                                        println!("无法删除已存在的文件：{}，错误：{}", output_path, e);
                                        continue;
                                    }
                                },
                                MessageDialogResult::No => output_path = get_output_path(input_path, &output_format.to_string(), false),
                                _=>{}
                            }
                        }

                        match convert_media(input_path, &output_path) {
                            Ok(_) => println!("转换成功：{} -> {}", input_path, output_path),
                            Err(e) => println!("任务转换失败：{}，错误：{}", input_path, e),
                        }
                    }
                }
            }
            AppEvent::ShowConifgWindow(idx)=>{
                self.show_config_window = true;
            }
            _ => unimplemented!(),
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
            show_config_window: false,
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
                    let name = item.then(Task::name).unwrap();
                    let supported_output_formats = item.then(Task::supported_output_formats);
                    let selected_output_format = item.then(Task::selected_output_format);
                    HStack::new(cx, |cx| {
                        Label::new(cx, name);
                        // ComboBox::new(cx, supported_output_formats, selected_output_format)
                        //     .on_select(move |cx, selected_format| {
                        //         cx.emit(AppEvent::ChangeOutputFormat(index, selected_format));
                        //     });
                        Button::new(cx, |cx| Label::new(cx, "Config")).on_press(move |cx|{
                            cx.emit(AppEvent::ShowConifgWindow(index));
                        });

                    });
                    // TaskItemRow::new(cx, AppData::tasks.map_ref(move |tasks| &tasks[index]).get());
                });
            });

            Binding::new(cx,AppData::show_config_window,|cx,show_config_window|{

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

fn get_output_path(input_path: &str, new_ext: &str, overwrite: bool) -> String {
    let path = Path::new(input_path);
    let stem = path.file_stem().unwrap_or_default().to_string_lossy();
    let parent = path.parent().unwrap_or_else(|| Path::new(""));

    let mut output_path = parent.join(format!("{}_converted.{}", stem, new_ext));
    let mut count = 1;

    if !overwrite{
        while output_path.exists() {
            output_path = parent.join(format!("{}_converted_{}.{}", stem, count, new_ext));
            count += 1;
        }
    }

    output_path.to_string_lossy().to_string()
}

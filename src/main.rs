use app_data::AppData;
use app_event::AppEvent;
use media_format::{Audio, MediaFormat, Video};
use task::Task;
use vizia::prelude::*;

mod app_data;
mod app_event;
mod combobox;
mod config_window;
mod media_format;
mod task;
mod utils;

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

        cx.add_stylesheet(include_style!("src/light_theme.css"))
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
                        VStack::new(cx, |cx| {
                            Label::new(cx, input_path);
                            Label::new(cx, output_path);
                        });
                        // ComboBox::new(cx, supported_output_formats, selected_output_format)
                        //     .on_select(move |cx, selected_format| {
                        //         cx.emit(AppEvent::ChangeOutputFormat(index, selected_format));
                        //     });
                        Button::new(cx, |cx| Label::new(cx, "Config")).on_press(move |cx| {
                            cx.emit(AppEvent::ToggleConifgWindow(index));
                        });
                    });
                });
            });

            Binding::new(cx, AppData::show_config_window, |cx, is_show| {
                if is_show.get(cx) {
                    config_window::popup(cx);
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

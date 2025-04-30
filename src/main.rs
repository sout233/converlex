use controllers::main::{app_data::AppData, app_event::AppEvent};
use models::{
    media_format::{Audio, MediaFormat, Video},
    task::Task,
};
use views::windows::config_window;
use vizia::prelude::*;

mod controllers;
mod models;
mod utils;
mod views;

#[tokio::main]
async fn main() -> Result<(), ApplicationError> {
    Application::new(|cx| {
        AppData {
            indices: vec![],
            tasks: vec![],
            show_config_window: false,
            configuring_index: None,
        }
        .build(cx);

        cx.add_stylesheet(include_style!("src/views/styles/light_theme.css"))
            .expect("failed to load style");

        cx.add_stylesheet(include_style!("src/views/styles/style.css"))
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
                    HStack::new(cx, |cx| {
                        VStack::new(cx, |cx| {
                            Label::new(cx, input_path);
                            Label::new(cx, output_path);
                        }).alignment(Alignment::Left);

                        Button::new(cx, |cx| Label::new(cx, "Config")).on_press(move |cx| {
                            cx.emit(AppEvent::ToggleConifgWindow(index));
                        });
                    }).class("task-row");
                });
            }).class("task-list");

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

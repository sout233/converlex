use controllers::main::{app_data::AppData, app_event::AppEvent};
use models::{app_settings::AppSettings, task::Task};
use views::windows::task_config_window;
use vizia::prelude::*;

mod controllers;
mod models;
mod utils;
mod views;


#[tokio::main]
async fn main() -> Result<(), ApplicationError> {
    let app_settings = AppSettings::omg_default().await;

    Application::new(move |cx| {
        AppData {
            indices: vec![],
            tasks: vec![],
            show_config_window: false,
            configuring_index: None,
            settings: app_settings.clone(),
            show_settings_window: false,
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

                Button::new(cx, |cx| Label::new(cx, "Settings"))
                    .on_press(|ex| ex.emit(AppEvent::ToggleSettingsWindow));
            })
            .class("menu-btns-row");

            List::new(cx, AppData::indices, |cx, _, idx| {
                Binding::new(cx, idx, |cx, index| {
                    let index = index.get(cx);
                    let item = AppData::tasks.map_ref(move |tasks| &tasks[index]);
                    let input_path = item.then(Task::input_path);
                    let output_path = item.then(Task::output_path);
                    let progress = item.then(Task::progress);
                    HStack::new(cx, |cx| {
                        VStack::new(cx, |cx| {
                            Label::new(cx, input_path);
                            Label::new(cx, output_path);
                            ProgressBar::new(cx,progress,Orientation::Horizontal);
                        })
                        .alignment(Alignment::Left);

                        Button::new(cx, |cx| Label::new(cx, "Config")).on_press(move |cx| {
                            cx.emit(AppEvent::ToggleConifgWindow(index));
                        });
                    })
                    .class("task-row");

                    // HStack::new(cx, |_| {}).height(Pixels(10.0));
                });
            })
            .class("task-list");

            Binding::new(cx, AppData::show_config_window, |cx, is_show| {
                if is_show.get(cx) {
                    task_config_window::popup(cx);
                }
            });

            Binding::new(cx, AppData::show_settings_window, |cx, is_show| {
                if is_show.get(cx) {
                    views::windows::setting_window::new(cx);
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

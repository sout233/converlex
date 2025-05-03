use std::{collections::HashMap, sync::Arc};

use controllers::main::{
    app_data::{AppData, app_data_derived_lenses::configuring_taskid},
    app_event::AppEvent,
};
use models::{
    app_settings::AppSettings,
    task::{Task, task_derived_lenses::supported_output_formats},
};
use views::windows::task_config_window;
use vizia::{
    input::{self, MouseState},
    prelude::*,
};

mod controllers;
mod models;
mod utils;
mod views;

#[tokio::main]
async fn main() -> Result<(), ApplicationError> {
    let app_settings = AppSettings::omg_default().await;

    Application::new(move |cx| {
        AppData {
            show_config_window: false,
            configuring_taskid: None,
            settings: app_settings.clone(),
            show_settings_window: false,
            task_ids: vec![],
            tasks: HashMap::new(),
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

            List::new(cx, AppData::task_ids, |cx, _, idx| {
                Binding::new(cx, idx, |cx, index| {
                    Binding::new(cx, AppData::configuring_taskid, move |cx, id| {
                        let index = index.get(cx);
                        let index = Arc::new(index.clone());
                        let index4mapping = Arc::clone(&index);
                        let index4click = Arc::clone(&index);
    
                        let item =
                            AppData::tasks.map_ref(move |tasks| &tasks[&index4mapping.to_string()]);
    
                        let input_path = item.then(Task::input_path);
                        let output_path = item.then(Task::output_path);
                        let input_format = input_path
                            .map(|path| path.split('.').last().unwrap_or_default().to_string());
    
                        let progress = item.then(Task::progress);
    
                        let index4color = Arc::clone(&index);

                        let configuring_tid = id.get(cx).unwrap_or_default();
                        let bg_color = if configuring_tid == *index4color {
                            Color::from("#00c3ff18")
                        } else {
                            Color::from("#ffffff")
                        };
                        let class_name = if configuring_tid == *index4color {
                            "task-row-selected"
                        } else {
                            "task-row"
                        };

                        VStack::new(cx, move |cx| {
                            HStack::new(cx, |cx| {
                                VStack::new(cx, |cx| {
                                    HStack::new(cx, |cx| {
                                        Label::new(cx, input_format).class("badge-label");
                                        Label::new(cx, input_path).padding_left(Pixels(5.0));
                                    });

                                    HStack::new(cx, |cx| {
                                        Binding::new(
                                            cx,
                                            item.then(Task::selected_output_format),
                                            move |cx, selected| {
                                                let formats = item
                                                    .then(Task::supported_output_formats)
                                                    .get(cx)
                                                    .clone();
                                                let format = formats[selected.get(cx)].to_string();
                                                Label::new(cx, format).class("badge-label");
                                            },
                                        );
                                        Label::new(cx, output_path).padding_left(Pixels(5.0));
                                    });
                                })
                                .class("task-paths")
                                .alignment(Alignment::Left);

                                Button::new(cx, |cx| Label::new(cx, "Config")).on_press(
                                    move |cx| {
                                        cx.emit(AppEvent::ToggleConifgWindow((&index).to_string()));
                                    },
                                );
                            });
                            ProgressBar::new(cx, progress, Orientation::Horizontal);
                        })
                        .on_mouse_down(move |ex, button| {
                            if button == MouseButton::Left {
                                ex.emit(AppEvent::ToggleConifgWindow((&index4click).to_string()));
                            }
                        })
                        .class(class_name);
                    });

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

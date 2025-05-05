use std::{sync::Arc, task};

use vizia::{
    icons::{ICON_MENU_3, ICON_SELECTOR},
    prelude::*,
};

use crate::{
    controllers::main::{app_data::AppData, app_event::AppEvent},
    models::task::Task,
};

pub fn popup(cx: &mut Context) -> Handle<Window> {
    Window::popup(cx, true, |cx| {
        Binding::new(cx, AppData::configuring_taskid, |cx, task_id| {
            let task_id = task_id.get(cx);
            if let Some(task_id) = task_id {
                let task_id_arc = Arc::new(task_id.clone());
                let item = AppData::tasks.map_ref(move |tasks| &tasks[&task_id]);
                let supported_output_formats = item.then(Task::supported_output_formats);
                let selected_output_format = item.then(Task::selected_output_format);

                VStack::new(cx, |cx| {
                    HStack::new(cx, |cx| {
                        Label::new(cx, "Output Format").class("title");
                    })
                    .class("config-row");

                    let supported_output_formats_arc = Arc::new(supported_output_formats);
                    List::new(cx, supported_output_formats, move |cx, _, fmt| {
                        let task_id_for_binding = Arc::clone(&task_id_arc);
                        let formats_arc = Arc::clone(&supported_output_formats_arc);
                        Binding::new(cx, fmt, move |cx, format| {
                            let format = format.get(cx);
                            let format_name = format.as_any().get_ext();
                            let formats = formats_arc.get(cx);
                            let this_task_idx = formats
                                .iter()
                                .position(|f| f.as_any().get_ext() == format_name)
                                .unwrap_or(0);
                            let task_id = task_id_for_binding.clone();

                            let ex_class_name = if this_task_idx == selected_output_format.get(cx) {
                                "selected"
                            } else {
                                ""
                            };

                            HStack::new(cx, |cx| {
                                Label::new(cx, format_name);
                            })
                            .on_mouse_down(move |ex, button| {
                                if button == MouseButton::Left {
                                    ex.emit(AppEvent::ChangeOutputFormat(
                                        task_id.to_string(),
                                        this_task_idx,
                                    ));
                                }
                            })
                            .class("format-row")
                            .class(ex_class_name);
                        });
                    })
                    .class("format-list");
                })
                .class("format-selector-main");
            }
        })
    })
    .title("Converlex - Format Selector")
    .on_close(|cx| {
        cx.emit(AppEvent::FormatSelectorWindowClosing);
    })
}

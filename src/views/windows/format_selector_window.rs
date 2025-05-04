use std::{sync::Arc, task};

use vizia::prelude::*;

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

                List::new(cx, supported_output_formats, |cx, _, fmt| {
                    Binding::new(cx, fmt, |cx, format| {
                            let task_id_for_binding = Arc::clone(&task_id_arc);
                            let format = format.get(cx);
                            let format_name = format.as_any().get_ext();
                            let formats = supported_output_formats.get(cx);
                            let this_task_idx = formats
                                .iter()
                                .position(|f| f.as_any().get_ext() == format_name)
                                .unwrap_or(0);

                            HStack::new(cx, |cx| {
                                Label::new(cx, format_name);
                            })
                            .on_mouse_down(move |ex, button| {
                                if button == MouseButton::Left {
                                    ex.emit(AppEvent::ChangeOutputFormat(
                                        task_id_for_binding.to_string(),
                                        this_task_idx,
                                    ));
                                }
                            })
                            .class("format-row");
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

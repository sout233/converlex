use std::sync::Arc;

use vizia::prelude::*;

use crate::{
    controllers::main::{app_data::AppData, app_event::AppEvent},
    models::{convertible_format::FormatType, task::Task},
};

#[derive(Lens, Data, Clone)]
pub struct SelectorData {
    filter_text: String,
}

pub enum SelectorEvent {
    UpdateFilterText(String),
}

impl Model for SelectorData {
    fn event(&mut self, cx: &mut EventContext, event: &mut Event) {
        event.map(|selector_event, _| match selector_event {
            SelectorEvent::UpdateFilterText(text) => {
                self.filter_text.clone_from(text);
            }
        })
    }
}

pub fn popup(cx: &mut Context) -> Handle<Window> {
    SelectorData {
        filter_text: Default::default(),
    }
    .build(cx);

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
                        Label::new(cx, "Output Format").class("title").width(Stretch(1.0));
                        Textbox::new(cx, SelectorData::filter_text)
                            .on_edit(|ex, new_text| {
                                ex.emit(SelectorEvent::UpdateFilterText(new_text));
                            })
                            .placeholder("Search format")
                            .width(Pixels(200.0));
                    })
                    .class("config-row");

                    Binding::new(cx, SelectorData::filter_text, move |cx, filter_text| {
                        let supported_output_formats_arc = Arc::new(supported_output_formats);
                        let task_id_arc2 = Arc::clone(&task_id_arc);
                        let f = filter_text.get(cx);
                        List::new_filtered(
                            cx,
                            supported_output_formats,
                            move |item| {
                                if f.is_empty() {
                                    true
                                } else {
                                    item.to_string()
                                        .to_ascii_lowercase()
                                        .contains(&f.to_ascii_lowercase())
                                }
                            },
                            move |cx, _, fmt| {
                                let task_id_for_binding = Arc::clone(&task_id_arc2);
                                let formats_arc = Arc::clone(&supported_output_formats_arc);
                                Binding::new(cx, fmt, move |cx, format| {
                                    let format = format.get(cx);
                                    let format_name = format.as_any().get_ext();
                                    let format_decs =
                                        format.as_any().get_decs().unwrap_or_default();
                                    let formats = formats_arc.get(cx);
                                    let this_task_idx = formats
                                        .iter()
                                        .position(|f| f.as_any().get_ext() == format_name)
                                        .unwrap_or(0);
                                    let task_id = task_id_for_binding.clone();
                                    let format_type = match format.as_any().get_format_type(){
                                        FormatType::Audio(_) => "Audio",
                                        FormatType::Video(_) => "Video",
                                    };


                                    HStack::new(cx, |cx| {
                                        VStack::new(cx, |cx| {
                                            HStack::new(cx, |cx|{
                                                Label::new(cx, format_name).padding_right(Pixels(5.0)).class("h4");
                                                Label::new(cx, format_type).class("badge-label").class("lite");
                                            }).height(Auto);
                                            Label::new(cx, format_decs).class("p-decs");
                                        })
                                        .alignment(Alignment::Left);
                                    })
                                    .bind(selected_output_format, move |handle, res| {
                                        if res.get(&handle) == this_task_idx {
                                            handle.toggle_class("selected", true);
                                        } else {
                                            handle.toggle_class("selected", false);
                                        }
                                    })
                                    .on_mouse_down(move |ex, button| {
                                        if button == MouseButton::Left {
                                            ex.emit(AppEvent::ChangeOutputFormat(
                                                task_id.to_string(),
                                                this_task_idx,
                                            ));
                                        }
                                    })
                                    .class("format-row");
                                });
                            },
                        )
                        .class("format-list");
                    });
                })
                .class("format-selector-main");
            }
        })
    })
    .title("Converlex - Format Selector")
    .anchor_target(AnchorTarget::Window)
    .parent_anchor(Anchor::Center)
    .on_close(|cx| {
        cx.emit(AppEvent::FormatSelectorWindowClosing);
    })
}

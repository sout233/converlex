use std::sync::Arc;

use vizia::{icons::{ICON_MENU_3, ICON_SELECTOR}, prelude::*};

use crate::{
    controllers::main::{app_data::AppData, app_event::AppEvent},
    models::task::Task,
};

pub fn new(cx: &mut Context) -> Handle<VStack> {
    VStack::new(cx, |cx| {
        Binding::new(cx, AppData::configuring_taskid, |cx, index| {
            let index = index.get(cx);
            if let Some(index) = index {
                let index_clone = Arc::new(index.clone());
                let item = AppData::tasks.map_ref(move |tasks| &tasks[&index]);
                let input_path = item.then(Task::input_path);
                let output_path = item.then(Task::output_path);
                let supported_output_formats = item.then(Task::supported_output_formats);
                let selected_output_format_idx = item.then(Task::selected_output_format);
                let is_auto_rename = item.then(Task::auto_rename);
                let task_type = item.then(Task::task_type);
                let task_status = item.then(Task::status);

                VStack::new(cx, |cx| {
                    HStack::new(cx, |cx| {
                        Label::new(cx, "Input").padding_right(Pixels(10.0));

                        let input_index = Arc::clone(&index_clone);
                        Textbox::new(cx, input_path).width(Stretch(1.0)).on_edit(
                            move |cx, new_input| {
                                cx.emit(AppEvent::UpdateTask(
                                    (&input_index).to_string(),
                                    Task {
                                        input_path: new_input.to_string(),
                                        output_path: output_path.get(cx).clone(),
                                        supported_output_formats: supported_output_formats
                                            .get(cx)
                                            .clone(),
                                        selected_output_format: selected_output_format_idx.get(cx),
                                        auto_rename: is_auto_rename.get(cx),
                                        progress: 0.0,
                                        task_type: task_type.get(cx).clone(),
                                        status: task_status.get(cx).clone(),
                                    },
                                ));
                            },
                        );
                    })
                    .class("config-row");
                    HStack::new(cx, |cx| {
                        Label::new(cx, "Output").padding_right(Pixels(10.0));

                        let output_index = Arc::clone(&index_clone);
                        Textbox::new(cx, output_path)
                            .width(Stretch(1.0))
                            .on_edit(move |cx, new_output| {
                                cx.emit(AppEvent::UpdateTask(
                                    (&output_index).to_string(),
                                    Task {
                                        output_path: new_output.to_string(),
                                        input_path: input_path.get(cx).clone(),
                                        supported_output_formats: supported_output_formats
                                            .get(cx)
                                            .clone(),
                                        selected_output_format: selected_output_format_idx.get(cx),
                                        auto_rename: is_auto_rename.get(cx),
                                        progress: 0.0,
                                        task_type: task_type.get(cx).clone(),
                                        status: task_status.get(cx).clone(),
                                    },
                                ));
                            })
                            .disabled(is_auto_rename);
                        HStack::new(cx, |cx| {
                            Label::new(cx, "Auto Rename").padding_right(Pixels(5.0));

                            let checkbox_index = Arc::clone(&index_clone);
                            Checkbox::new(cx, is_auto_rename).on_toggle(move |cx| {
                                cx.emit(AppEvent::ToggleAutoRename(checkbox_index.to_string()));
                            });
                        })
                        .class("auto-rename-checkbox");
                    })
                    .class("config-row");
                    HStack::new(cx, |cx| {
                        Label::new(cx, "Output Format").width(Stretch(1.0));

                        // let pick_index = Arc::clone(&index_clone);
                        // PickList::new(
                        //     cx,
                        //     supported_output_formats,
                        //     selected_output_format_idx,
                        //     true,
                        // )
                        // .alignment(Alignment::Right)
                        // .width(Pixels(100.0))
                        // .on_select(move |cx, selected_format| {
                        //     cx.emit(AppEvent::ChangeOutputFormat(
                        //         pick_index.to_string(),
                        //         selected_format,
                        //     ));
                        // });

                        let pick_index2 = Arc::clone(&index_clone);
                        Button::new(cx, |cx| {
                            HStack::new(cx, |cx| {
                                Label::new(cx, "").bind(
                                    supported_output_formats,
                                    move |handle, list| {
                                        handle.bind(selected_output_format_idx, move |handle, sel| {
                                            let selected_index = sel.get(&handle);
                                            let list_len = list.map(|list| list.len()).get(&handle);
                                            if selected_index < list_len {
                                                handle.text(list.idx(selected_index));
                                            } else {
                                                handle.text("");
                                            }
                                        });
                                    },
                                );
                                Svg::new(cx, ICON_SELECTOR).padding_left(Pixels(5.0));
                            }).alignment(Alignment::Center)
                        })
                        .on_press(move |cx| {
                            cx.emit(AppEvent::ToggleFormatSelectorWindow(
                                pick_index2.to_string(),
                            ));
                        });
                    })
                    .class("config-row");
                });
            }
        });
    })
}

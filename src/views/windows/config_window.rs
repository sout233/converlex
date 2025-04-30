use vizia::prelude::*;

use crate::{
    controllers::main::app_data::AppData, controllers::main::app_event::AppEvent,
    models::task::Task,
};

pub fn popup(cx: &mut Context) -> Handle<Window> {
    Window::popup(cx, true, |cx| {
        Binding::new(cx, AppData::configuring_index, |cx, index| {
            let index = index.get(cx);
            if let Some(index) = index {
                let item = AppData::tasks.map_ref(move |tasks| &tasks[index]);
                let input_path = item.then(Task::input_path);
                let output_path = item.then(Task::output_path);
                let supported_output_formats = item.then(Task::supported_output_formats);
                let selected_output_format = item.then(Task::selected_output_format);
                let is_auto_rename = item.then(Task::auto_rename);
                let is_done = item.then(Task::done);

                VStack::new(cx, |cx| {
                    HStack::new(cx, |cx| {
                        Label::new(cx, "Input").padding_right(Pixels(10.0));
                        // Label::new(cx, name);
                        Textbox::new(cx, input_path).width(Stretch(1.0)).on_edit(
                            move |cx, new_input| {
                                cx.emit(AppEvent::UpdateTask(
                                    index,
                                    Task {
                                        input_path: new_input.to_string(),
                                        output_path: output_path.get(cx).clone(),
                                        done: is_done.get(cx),
                                        supported_output_formats: supported_output_formats
                                            .get(cx)
                                            .clone(),
                                        selected_output_format: selected_output_format.get(cx),
                                        auto_rename: is_auto_rename.get(cx),
                                        progress: 0.0,
                                    },
                                ));
                            },
                        );
                    })
                    .class("config-row");
                    HStack::new(cx, |cx| {
                        Label::new(cx, "Output").padding_right(Pixels(10.0));
                        Textbox::new(cx, output_path)
                            .width(Stretch(1.0))
                            .on_edit(move |cx, new_output| {
                                cx.emit(AppEvent::UpdateTask(
                                    index,
                                    Task {
                                        output_path: new_output.to_string(),
                                        input_path: input_path.get(cx).clone(),
                                        done: is_done.get(cx),
                                        supported_output_formats: supported_output_formats
                                            .get(cx)
                                            .clone(),
                                        selected_output_format: selected_output_format.get(cx),
                                        auto_rename: is_auto_rename.get(cx),
                                        progress: 0.0,
                                    },
                                ));
                            })
                            .disabled(is_auto_rename);
                        HStack::new(cx, |cx| {
                            Label::new(cx, "Auto Rename");
                            Checkbox::new(cx, is_auto_rename).on_toggle(move |cx| {
                                cx.emit(AppEvent::ToggleAutoRename(index));
                            });
                        })
                        .class("auto-rename-checkbox");
                    })
                    .class("config-row");
                    HStack::new(cx, |cx| {
                        Label::new(cx, "Output Format").width(Stretch(1.0));
                        PickList::new(cx, supported_output_formats, selected_output_format, true)
                            .alignment(Alignment::Right)
                            .width(Pixels(100.0))
                            .on_select(move |cx, selected_format| {
                                cx.emit(AppEvent::ChangeOutputFormat(index, selected_format));
                            });
                        // ComboBox::new(
                        //     cx,
                        //     supported_output_formats,
                        //     selected_output_format,
                        // )
                        // .alignment(Alignment::Right)
                        // .width(Pixels(100.0))
                        // .on_select(
                        //     move |cx, selected_format| {
                        //         cx.emit(AppEvent::ChangeOutputFormat(
                        //             index,
                        //             selected_format,
                        //         ));
                        //     },
                        // );
                    })
                    .class("config-row");
                });
            }
        });
    })
    .title("Converlex - Config")
    .on_close(|cx| {
        cx.emit(AppEvent::ConfigWindowClosing);
    })
}

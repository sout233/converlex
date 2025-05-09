use std::{sync::Arc, task};

use vizia::{
    icons::{ICON_MENU_3, ICON_SELECTOR},
    prelude::*,
};

use crate::{
    controllers::main::{app_data::AppData, app_event::AppEvent},
    models::{
        convertible_format::FormatType,
        task::{ Task, TaskType},
    },
    unwrap_or_msgbox, utils::ffmpeg_wrapper::FfmpegTask,
};

pub fn new(cx: &mut Context) -> Handle<VStack> {
    VStack::new(cx, |cx| {
        Binding::new(cx, AppData::configuring_taskid, |cx, tid| {
            let taskid_opt = tid.get(cx);
            if let Some(taskid) = taskid_opt {
                let taskid_clone = Arc::new(taskid.clone());
                let item = AppData::tasks.map_ref(move |tasks| &tasks[&taskid]);
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

                        let input_index = Arc::clone(&taskid_clone);
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

                        let output_index = Arc::clone(&taskid_clone);
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

                            let checkbox_index = Arc::clone(&taskid_clone);
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

                        let pick_index2 = Arc::clone(&taskid_clone);
                        Button::new(cx, |cx| {
                            HStack::new(cx, |cx| {
                                let supported_output_formats = supported_output_formats.clone();
                                let selected_output_format_idx = selected_output_format_idx.clone();
                                Label::new(cx, "").bind(
                                    supported_output_formats,
                                    move |handle, list| {
                                        handle.bind(
                                            selected_output_format_idx,
                                            move |handle, sel| {
                                                let selected_index = sel.get(&handle);
                                                let list_len =
                                                    list.map(|list| list.len()).get(&handle);
                                                if selected_index < list_len {
                                                    handle.text(list.idx(selected_index));
                                                } else {
                                                    handle.text("");
                                                }
                                            },
                                        );
                                    },
                                );
                                Svg::new(cx, ICON_SELECTOR).padding_left(Pixels(5.0));
                            })
                            .alignment(Alignment::Center)
                        })
                        .on_press(move |cx| {
                            cx.emit(AppEvent::ToggleFormatSelectorWindow(
                                pick_index2.to_string(),
                            ));
                        });
                    })
                    .class("config-row");

                    let task_type = item.then(Task::task_type);
                    let task_type = task_type.map(|tt| match tt {
                        TaskType::Ffmpeg(a) => a.clone(),
                    });

                    let ab = task_type.map(|tt|{
                        tt.audio_bitrate.unwrap_or(0)
                    });

                    let taskid = Arc::new(tid);
                    let taskid = taskid.get(cx).unwrap();
                    Textbox::new(cx, ab).on_edit(move |ex,new_text|{
                        let a = new_text.parse::<u32>();
                        if let Ok(new_bitrate) = a{
                            ex.emit(AppEvent::ChangeAudioBitrate(taskid.to_string(),new_bitrate));
                        }else{
                            // ex.set_text("0");
                        }
                    });

                    // let videto_bitrate = item.then(Task::)
                    VStack::new(cx, |cx| {
                        let supported_output_formats = supported_output_formats.clone();
                        let selected_output_format_idx = selected_output_format_idx.clone();
                        let taskid = Arc::new(tid);

                        Binding::new(cx, item.then(Task::task_type), move |cx, t_type| {
                            let taskid = Arc::clone(&taskid);
                            Binding::new(
                                cx,
                                item.then(Task::selected_output_format),
                                move |cx, idx| {
                                    let task_type = t_type.get(cx);
                                    let idx = idx.get(cx);
                                    let selected_format = supported_output_formats.idx(idx);
                                    let taskid = Arc::clone(&taskid);
                                    Binding::new(cx, selected_format, move |cx, format_binding| {
                                        let format_type = format_binding.get(cx).get_type();

                                        match &format_type {
                                            FormatType::Audio(_) => match &task_type {
                                                TaskType::Ffmpeg(ffmpeg_task) => {
                                                    let audio_bitrate = ffmpeg_task.audio_bitrate;
                                                    let placeholder = if audio_bitrate.is_none() {
                                                        "Undef"
                                                    } else {
                                                        ""
                                                    };

                                                    let audio_bitrate = t_type.map(|tt| match tt {
                                                        TaskType::Ffmpeg(ffmpeg_task) => {
                                                            if let Some(br) =
                                                                ffmpeg_task.audio_bitrate
                                                            {
                                                                br.to_string()
                                                            } else {
                                                                String::default()
                                                            }
                                                        }
                                                    });

                                                    HStack::new(cx, |cx| {
                                                        Label::new(cx, "Audio Bitrate")
                                                            .width(Stretch(1.0));
                                                        let taskid = Arc::clone(&taskid);
                                                        let taskid = Arc::new(unwrap_or_msgbox!(taskid.get(cx)));
                                                        Textbox::new(cx, audio_bitrate)
                                                            // .placeholder(placeholder)
                                                            .on_edit(move |ex,new_txt|{
                                                                let a = new_txt.parse::<u32>();
                                                                if let Ok(new_bitrate) = a{
                                                                    ex.emit(AppEvent::ChangeAudioBitrate(taskid.to_string(),new_bitrate));
                                                                }else{
                                                                    // ex.set_text("0");
                                                                }
                                                            }).width(Pixels(50.0));
                                                    });
                                                }
                                            },
                                            FormatType::Video(_) => {
                                                Label::new(cx, "Video Bitrate").width(Stretch(1.0));
                                            }
                                        }
                                    });
                                },
                            );
                        });
                    })
                    .class("config-row");
                });
            }
        });
    })
}

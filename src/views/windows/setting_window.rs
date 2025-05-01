use std::path::{Path, PathBuf};

use vizia::prelude::*;

use crate::{
    controllers::main::{app_data::AppData, app_event::AppEvent},
    models::app_settings::AppSettings,
    utils::ffmpeg_wrapper::FfmpegEntry,
};

pub fn new(cx: &mut Context) -> Handle<Window> {
    Window::new(cx, |cx: &mut Context| {
        VStack::new(cx, |cx| {
            Label::new(cx, "Settings").class("title");
            HStack::new(cx, |cx| {
                Label::new(cx, "FFmpeg Path").padding_right(Pixels(10.0));
                Textbox::new(cx, AppSettings::ffmpeg_entry.map(|fe|{
                    match fe {
                        Some(a) => a.to_string(),
                        None => "".to_string(),
                    }
                })).on_edit(move |cx, new_path| {
                    cx.emit(AppEvent::UpdateFfmpegEntry(Some(FfmpegEntry::Path(PathBuf::from(new_path)))));
                }).width(Stretch(1.0));
            })
            .class("config-row");
        });
    })
}

use std::path::PathBuf;

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

                let ffmpeg_entry_binding = AppData::settings
                    .then(AppSettings::ffmpeg_entry)
                    .map(|opt| opt.as_ref().map(|e| e.to_string()).unwrap_or_default());

                Textbox::new(cx, ffmpeg_entry_binding)
                    .on_edit(move |cx, new_text| {
                        let new_entry = if new_text.is_empty() {
                            None
                        } else {
                            Some(FfmpegEntry::Path(PathBuf::from(new_text)))
                        };

                        cx.emit(AppEvent::UpdateFfmpegEntry(new_entry));
                    })
                    .width(Stretch(1.0));
            })
            .class("setting-row");
        })
        .class("settings-window-content");
    })
    .title("Converlex - Settings")
}

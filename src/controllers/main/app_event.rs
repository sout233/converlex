use crate::{models::{app_settings::AppSettings, task::Task}, utils::ffmpeg_wrapper::FfmpegEntry};

pub enum AppEvent {
    AddTask(Option<String>),
    RemoveTask(usize),
    UpdateTask(usize, Task),
    ToggleAutoRename(usize),
    ChangeOutputFormat(usize, usize),
    StartConvert,
    RemoveAll,
    ToggleConifgWindow(usize),
    ConfigWindowClosing,
    UpdateProgress(usize, f32),
    MarkDone(usize),
    UpdateAppSettings(Box<dyn FnOnce(&mut AppSettings) + Send>),
    UpdateFfmpegEntry(Option<FfmpegEntry>),
    ToggleSettingsWindow,
}
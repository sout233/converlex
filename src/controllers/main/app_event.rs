use crate::{models::{app_settings::AppSettings, task::Task}, utils::ffmpeg_wrapper::FfmpegEntry};

type TaskId = String;

pub enum AppEvent {
    AddTask(Option<String>),
    RemoveTask(TaskId),
    UpdateTask(String, Task),
    ToggleAutoRename(TaskId),
    ChangeOutputFormat(TaskId, usize),
    StartConvert,
    RemoveAll,
    ToggleConifgWindow(TaskId),
    ConfigWindowClosing,
    UpdateProgress(TaskId, f32),
    MarkDone(TaskId),
    UpdateAppSettings(Box<dyn FnOnce(&mut AppSettings) + Send>),
    UpdateFfmpegEntry(Option<FfmpegEntry>),
    ToggleSettingsWindow,
}
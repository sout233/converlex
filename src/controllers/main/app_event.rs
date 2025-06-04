use crate::{models::{app_settings::AppSettings, task::Task}, utils::ffmpeg_wrapper::FfmpegEntry};

type TaskId = String;

pub enum AppEvent {
    AddTask(Option<String>),
    RemoveTask(TaskId),
    UpdateTask(String, Task),
    ToggleAutoRename(TaskId),
    ChangeOutputFormat(TaskId, usize),
    StartConvert(Option<Vec<TaskId>>),
    RemoveAll,
    ToggleConifg(TaskId),
    ConfigWindowClosing,
    UpdateProgress(TaskId, f32),
    MarkDone(TaskId,bool),
    UpdateAppSettings(Box<dyn FnOnce(&mut AppSettings) + Send>),
    UpdateFfmpegEntry(Option<FfmpegEntry>),
    ToggleSettingsWindow,
    ToggleFormatSelectorWindow(TaskId),
    FormatSelectorWindowClosing,
    ChangeAudioBitrate(TaskId,Option<u32>),
    ChangeVideoBitrate(TaskId,Option<u32>),
}
use crate::utils::ffmpeg_wrapper::{self, FfmpegEntry};
use vizia::prelude::*;

#[derive(Lens, Debug, Clone, Data)]
pub struct AppSettings {
    pub ffmpeg_entry: Option<FfmpegEntry>,
}

impl AppSettings {
    pub fn new() -> Self {
        Self { ffmpeg_entry: None }
    }

    pub fn with_ffmpeg_entry(ffmpeg_entry: FfmpegEntry) -> Self {
        Self {
            ffmpeg_entry: Some(ffmpeg_entry),
        }
    }

    // pub fn ffmpeg_entry(&self) -> &Option<FfmpegEntry> {
    //     &self.ffmpeg_entry
    // }

    pub async fn omg_default() -> AppSettings {
        first_run_init().await
    }
}

async fn first_run_init() -> AppSettings {
    let ffmpeg_entry = ffmpeg_wrapper::find_ffmpeg()
        .await
        .unwrap_or(FfmpegEntry::Env);

    let settings = AppSettings::with_ffmpeg_entry(ffmpeg_entry);

    settings
}

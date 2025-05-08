use vizia::prelude::*;

use crate::utils::ffmpeg_wrapper::{FfmpegEntry, FfmpegTask};

use super::convertible_format::ConvertibleFormat;
use std::sync::Arc;

#[derive(Lens, Data, Clone, Debug)]
pub struct Task {
    pub input_path: String,
    pub output_path: String,
    // pub config: ConvertConfig,
    pub supported_output_formats: Vec<Arc<dyn ConvertibleFormat>>,
    pub selected_output_format: usize,
    pub auto_rename: bool,
    pub progress: f32,
    pub task_type: TaskType,
    pub status: TaskStatus,
}

impl Task {
    pub fn new(
        input_path: String,
        output_path: String,
        supported_output_formats: Vec<Arc<dyn ConvertibleFormat>>,
        selected_output_format: usize,
        ffmpeg_entry: FfmpegEntry,
    ) -> Self {
        let output_format = supported_output_formats
            .get(selected_output_format)
            .unwrap()
            .clone();
        Self {
            input_path,
            output_path,
            supported_output_formats,
            selected_output_format,
            auto_rename: true,
            progress: 0.0,
            task_type: TaskType::Ffmpeg(FfmpegTask::new(ffmpeg_entry, output_format)),
            status: TaskStatus::Queued,
        }
    }
}

#[derive(Data, Clone, Debug, PartialEq)]
pub enum TaskType {
    Ffmpeg(FfmpegTask),
}

#[derive(Data, Clone, Debug, PartialEq)]
pub enum TaskStatus {
    Queued,
    Running,
    Done,
    Failed,
}

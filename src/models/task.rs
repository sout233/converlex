use vizia::prelude::*;

use super::{convertible_format::ConvertibleFormat, task_type::TaskType};
use std::sync::Arc;


#[derive(Lens, Data, Clone,Debug)]
pub struct Task {
    pub input_path: String,
    pub output_path: String,
    // pub config: ConvertConfig,
    pub done: bool,
    pub supported_output_formats: Vec<Arc<dyn ConvertibleFormat>>,
    pub selected_output_format: usize,
    pub auto_rename: bool,
    pub progress: f32,
    pub task_type: TaskType,
}


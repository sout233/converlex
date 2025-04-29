use crate::media_format::MediaFormat;
use vizia::prelude::*;


#[derive(Lens, Data, Clone)]
pub struct Task {
    pub input_path: String,
    pub output_path: String,
    // pub config: ConvertConfig,
    pub done: bool,
    pub supported_output_formats: Vec<MediaFormat>,
    pub selected_output_format: usize,
    pub auto_rename: bool,
}
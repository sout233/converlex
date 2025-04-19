use std::{path::Path, process::Command};

use crate::media_format::MediaFormat;


pub fn convert_media(input: &str, output: &str) -> Result<(), String> {
    let ffmpeg_path = "./ffmpeg.exe"; // 确保可执行文件打包进程序目录
    if !Path::new(ffmpeg_path).exists() {
        return Err("找不到 ffmpeg.exe".to_string());
    }

    let status = Command::new(ffmpeg_path)
        .args(["-i", input, output])
        .status()
        .map_err(|e| format!("启动失败: {}", e))?;

    if status.success() {
        Ok(())
    } else {
        Err("ffmpeg 转换失败".into())
    }
}

pub fn get_output_path(input_path: &str, new_ext: &MediaFormat, overwrite: bool) -> String {
    let path = Path::new(input_path);
    let stem = path.file_stem().unwrap_or_default().to_string_lossy();
    let parent = path.parent().unwrap_or_else(|| Path::new(""));

    let mut output_path = parent.join(format!("{}_converted.{}", stem, new_ext));
    let mut count = 1;

    if !overwrite {
        while output_path.exists() {
            output_path = parent.join(format!("{}_converted_{}.{}", stem, count, new_ext));
            count += 1;
        }
    }

    output_path.to_string_lossy().to_string()
}

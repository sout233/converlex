use std::{io::{BufRead, BufReader}, path::Path, process::{Command, Stdio}};

use crate::models::convertible_format::ConvertibleFormat;



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


pub fn convert_media_with_progress<F>(input: &str, output: &str, mut on_progress: F) -> Result<(), String>
where
    F: FnMut(f32),
{
    let ffmpeg_path = "./ffmpeg.exe";

    if !Path::new(ffmpeg_path).exists() {
        return Err("找不到 ffmpeg.exe".to_string());
    }

    // 第一步：先获取总时长
    let duration = get_media_duration(input)?;
    if duration == 0.0 {
        return Err("无法解析媒体时长".to_string());
    }

    let mut child = Command::new(ffmpeg_path)
        .args(["-i", input, "-y", output])
        .stderr(Stdio::piped())
        .stdout(Stdio::null())
        .spawn()
        .map_err(|e| format!("无法启动 ffmpeg：{}", e))?;

    if let Some(stderr) = child.stderr.take() {
        let reader = BufReader::new(stderr);
        for line in reader.lines() {
            if let Ok(line) = line {
                if let Some(time_str) = line.split("time=").nth(1) {
                    if let Some(time_part) = time_str.split_whitespace().next() {
                        if let Ok(seconds) = parse_ffmpeg_time(time_part) {
                            let progress = seconds / duration;
                            on_progress(progress.clamp(0.0, 1.0));
                        }
                    }
                }
            }
        }
    }

    let status = child.wait().map_err(|e| format!("等待进程失败：{}", e))?;
    if status.success() {
        Ok(())
    } else {
        Err("转换失败".into())
    }
}


fn parse_ffmpeg_time(time_str: &str) -> Result<f32, std::num::ParseFloatError> {
    let parts: Vec<&str> = time_str.split(':').collect();
    if parts.len() != 3 {
        return Ok(0.0); // 默认错误处理
    }
    let hours: f32 = parts[0].parse()?;
    let minutes: f32 = parts[1].parse()?;
    let seconds: f32 = parts[2].parse()?;
    Ok(hours * 3600.0 + minutes * 60.0 + seconds)
}

fn get_media_duration(input: &str) -> Result<f32, String> {
    let output = Command::new("./ffmpeg.exe")
        .args(["-i", input])
        .stderr(Stdio::piped())
        .output()
        .map_err(|e| format!("获取时长失败：{}", e))?;

    let stderr = String::from_utf8_lossy(&output.stderr);
    for line in stderr.lines() {
        if line.contains("Duration:") {
            if let Some(dur_str) = line.split("Duration: ").nth(1) {
                if let Some(time_str) = dur_str.split(',').next() {
                    return parse_ffmpeg_time(time_str).map_err(|e| e.to_string());
                }
            }
        }
    }

    Err("未找到媒体时长".into())
}


pub fn get_output_path(input_path: &str, new_format: &dyn ConvertibleFormat, overwrite: bool) -> String {
    let path = Path::new(input_path);
    let stem = path.file_stem().unwrap_or_default().to_string_lossy();
    let parent = path.parent().unwrap_or_else(|| Path::new(""));

    let mut output_path = parent.join(format!("{}_converted.{}", stem, new_format.get_ext()));
    let mut count = 1;

    if !overwrite {
        while output_path.exists() {
            output_path = parent.join(format!("{}_converted_{}.{}", stem, count, new_format.get_ext()));
            count += 1;
        }
    }

    output_path.to_string_lossy().to_string()
}

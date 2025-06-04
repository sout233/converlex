use futures_util::future;
use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::str::FromStr;
use std::sync::Arc;
use tokio::process::Command;
use tokio::sync::mpsc;
use vizia::prelude::*;

use crate::err_msgbox;
use crate::models::convertible_format::ConvertibleFormat;

#[derive(Debug, Clone, Data)]
pub struct FfmpegTask {
    pub ffmpeg_entry: FfmpegEntry,
    pub input: Option<PathBuf>,
    pub output: Option<PathBuf>,
    pub output_format: Arc<dyn ConvertibleFormat>,
    pub video_bitrate: Option<u32>,
    pub audio_bitrate: Option<u32>,
    pub resolution: Option<(u32, u32)>,
    pub frame_rate: Option<u32>,
    pub sample_rate: Option<u32>,
    pub extra_args: Vec<String>,
}

impl PartialEq for FfmpegTask {
    fn eq(&self, other: &Self) -> bool {
        self.ffmpeg_entry == other.ffmpeg_entry
            && self.input == other.input
            && self.output == other.output
            && self.output_format.get_ext() == other.output_format.get_ext()
            && self.video_bitrate == other.video_bitrate
            && self.audio_bitrate == other.audio_bitrate
            && self.resolution == other.resolution
            && self.frame_rate == other.frame_rate
            && self.sample_rate == other.sample_rate
            && self.extra_args == other.extra_args
    }
}

#[derive(Debug, Clone, Data, PartialEq)]
pub enum FfmpegEntry {
    Path(PathBuf),
    Env,
}

impl ToString for FfmpegEntry {
    fn to_string(&self) -> String {
        match self {
            FfmpegEntry::Path(path) => path.to_string_lossy().to_string(),
            FfmpegEntry::Env => "ffmpeg".to_string(),
        }
    }
}

impl Into<PathBuf> for FfmpegEntry {
    fn into(self) -> PathBuf {
        match self {
            FfmpegEntry::Path(path) => path,
            FfmpegEntry::Env => PathBuf::from("ffmpeg"),
        }
    }
}

impl FromStr for FfmpegEntry {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.is_empty() {
            return Err("Empty string".to_string());
        }

        let path = PathBuf::from(s);
        if path.exists() {
            Ok(FfmpegEntry::Path(path))
        } else {
            Ok(FfmpegEntry::Env)
        }
    }
}

#[allow(dead_code)]
impl FfmpegTask {
    pub fn new(ffmpeg_entry: FfmpegEntry, output_format: Arc<dyn ConvertibleFormat>) -> Self {
        Self {
            ffmpeg_entry,
            input: None,
            output: None,
            output_format: output_format,
            video_bitrate: None,
            audio_bitrate: None,
            resolution: None,
            frame_rate: None,
            sample_rate: None,
            extra_args: vec![],
        }
    }

    pub fn input(mut self, path: impl Into<PathBuf>) -> Self {
        self.input = Some(path.into());
        self
    }

    pub fn output(mut self, path: impl Into<PathBuf>) -> Self {
        self.output = Some(path.into());
        self
    }

    pub fn video_bitrate(mut self, kbps: Option<u32>) -> Self {
        self.video_bitrate = kbps;
        self
    }

    pub fn audio_bitrate(mut self, kbps: Option<u32>) -> Self {
        self.audio_bitrate = kbps;
        self
    }

    pub fn resolution(mut self, width: u32, height: u32) -> Self {
        self.resolution = Some((width, height));
        self
    }

    pub fn frame_rate(mut self, fps: u32) -> Self {
        self.frame_rate = Some(fps);
        self
    }

    pub fn sample_rate(mut self, rate: u32) -> Self {
        self.sample_rate = Some(rate);
        self
    }

    pub fn extra_arg(mut self, arg: impl Into<String>) -> Self {
        self.extra_args.push(arg.into());
        self
    }

    pub fn build(self) -> Result<(PathBuf, Vec<String>), String> {
        println!("{:?}",self.clone());
        let input = self.input.ok_or("Missing input path")?;
        let output = self.output.ok_or("Missing output path")?;

        let mut args = vec![
            "-y".into(),
            "-i".into(),
            input.to_string_lossy().into_owned(),
        ];

        if let Some(b) = self.video_bitrate {
            args.push("-b:v".into());
            args.push(format!("{}k", b));
        }

        if let Some(b) = self.audio_bitrate {
            args.push("-b:a".into());
            args.push(format!("{}k", b));
        }

        if let Some((w, h)) = self.resolution {
            args.push("-s".into());
            args.push(format!("{}x{}", w, h));
        }

        if let Some(fps) = self.frame_rate {
            args.push("-r".into());
            args.push(fps.to_string());
        }

        if let Some(sr) = self.sample_rate {
            args.push("-ar".into());
            args.push(sr.to_string());
        }

        // args.push("-f".into());
        // args.push(self.output_format.to_string().to_string());

        args.extend(self.extra_args);

        args.push(output.to_string_lossy().into_owned());

        Ok((output, args))
    }

    pub async fn run_with_progress(&self, task_id: String, tx: mpsc::UnboundedSender<ProgressMsg>) {
        match &self.clone().build() {
            Ok((_output, args)) => {
                let task_id = Arc::new(task_id);

                let callback = {
                    let task_id = Arc::clone(&task_id);
                    let tx = tx.clone();
                    move |progress: f32| {
                        let _ = tx.send(ProgressMsg::Progress {
                            task_id: task_id.as_str().to_string(),
                            progress,
                        });
                    }
                };

                let ffmpeg_entry = self.ffmpeg_entry.clone();
                match run_ffmpeg_command_with_progress(
                    ffmpeg_entry,
                    task_id.to_string(),
                    args.clone(),
                    callback,
                )
                .await
                {
                    Ok(_) => {
                        let _ = tx.send(ProgressMsg::Done {
                            task_id: task_id.to_string(),
                        });
                        println!("[Task {task_id}] ✅ Task completed.");
                    }
                    Err(e) => {
                        let _ = tx.send(ProgressMsg::Error {
                            task_id: task_id.to_string(),
                            error: e.to_string(),
                        });
                        eprintln!("[Task {task_id}] ❌ Error: {:?}", e);
                    }
                }
            }
            Err(e) => {
                let e = format!("[Task {task_id}] ❌ Invalid config: {}", e);
                eprintln!("{e}");
                err_msgbox!(e);
            }
        }
    }
}

#[derive(Debug, Clone)]
pub enum ProgressMsg {
    Progress { task_id: String, progress: f32 },
    Done { task_id: String },
    Error { task_id: String, error: String },
}

/// 并发处理多个任务
pub async fn run_batch(
    tasks: Vec<(String, FfmpegTask)>,
    tx: mpsc::UnboundedSender<ProgressMsg>,
) -> anyhow::Result<()> {
    let futures = tasks.into_iter().map(|(id, task)| {
        let tx_clone = tx.clone();
        tokio::spawn(async move {
            task.run_with_progress(id, tx_clone).await;
        })
    });

    future::join_all(futures).await;
    Ok(())
}
async fn run_ffmpeg_command_with_progress<F>(
    entity: FfmpegEntry,
    id: String,
    args: Vec<String>,
    mut progress_cb: F,
) -> anyhow::Result<()>
where
    F: FnMut(f32) + Send + 'static,
{
    use tokio::io::AsyncReadExt;

    println!(
        "[Task {id}] ▶ Running: {} {}",
        entity.to_string(),
        args.join(" ")
    );

    let mut cmd = Command::new(entity.to_string());
    cmd.args(&args)
        .stderr(Stdio::piped())
        .stdout(Stdio::null())
        .stdin(Stdio::null());

    let mut child = cmd.spawn().map_err(|e| {
        anyhow::anyhow!(
            "Failed to spawn ffmpeg process: {}\nCommand: {} {}",
            e,
            entity.to_string(),
            args.join(" ")
        )
    })?;

    let mut stderr = child.stderr.take().expect("Failed to capture stderr");
    let mut buffer = vec![0u8; 4096];
    let mut raw = Vec::new();
    let mut duration_secs: Option<f32> = None;
    let mut full_stderr = String::new();

    loop {
        let n = stderr.read(&mut buffer).await?;
        if n == 0 {
            break;
        }
        raw.extend_from_slice(&buffer[..n]);

        while let Some(pos) = raw.iter().position(|&b| b == b'\r') {
            let line = raw.drain(..=pos).collect::<Vec<_>>();
            if let Ok(text) = String::from_utf8(line) {
                if text.to_lowercase().contains("error") || text.to_lowercase().contains("err") {
                    full_stderr.push_str(&text);
                }

                if text.contains("Duration:") {
                    if let Some(dur) = parse_duration(&text) {
                        duration_secs = Some(dur);
                        println!("[Task {id}] 🎬 Duration = {}s", dur);
                    }
                } else if text.contains("time=") {
                    if let Some(current_time) = parse_progress_time(&text) {
                        if let Some(total) = duration_secs {
                            let ratio = (current_time / total).min(1.0);
                            progress_cb(ratio);
                        }
                    }
                }
            }
        }
    }

    let status = child.wait().await?;
    if !status.success() {
        return Err(anyhow::anyhow!(
            "ffmpeg exited with status {}\nFull stderr:\n{}",
            status,
            full_stderr
        ));
    }

    Ok(())
}

fn parse_duration(line: &str) -> Option<f32> {
    let start = line.find("Duration: ")? + 10;
    let end = line[start..].find(',')? + start;
    let time_str = &line[start..end];
    parse_time_str(time_str)
}

fn parse_progress_time(line: &str) -> Option<f32> {
    let start = line.find("time=")? + 5;
    let end = line[start..].find(' ')? + start;
    let time_str = &line[start..end];
    parse_time_str(time_str)
}

fn parse_time_str(s: &str) -> Option<f32> {
    let parts: Vec<&str> = s.trim().split(':').collect();
    if parts.len() != 3 {
        return None;
    }
    let h = parts[0].parse::<f32>().ok()?;
    let m = parts[1].parse::<f32>().ok()?;
    let s = parts[2].parse::<f32>().ok()?;
    Some(h * 3600.0 + m * 60.0 + s)
}

pub async fn find_ffmpeg() -> Option<FfmpegEntry> {
    #[cfg(unix)]
    let which_cmd = Command::new("which").arg("ffmpeg").output();

    #[cfg(windows)]
    let which_cmd = Command::new("where").arg("ffmpeg").output();

    // 1. use cmd to get path
    if let Ok(output) = which_cmd.await {
        let stdout = str::from_utf8(&output.stdout);
        if stdout.is_ok() && output.status.success() {
            let path = stdout.unwrap();
            println!("{path}");
            let first_path = path.lines().next();
            if let Some(path) = first_path {
                return Some(FfmpegEntry::Path(path.trim().into()));
            }
        }
    }

    // 2. or use ffmpeg from current directory
    if let Ok(exe_path) = std::env::current_exe() {
        let exe_dir = exe_path.parent().unwrap_or(Path::new("."));
        println!("exe_dir: {:?}", exe_dir);
        #[cfg(target_os = "windows")]
        let ffmpeg_path = exe_dir.join("ffmpeg.exe");
        #[cfg(not(target_os = "windows"))]
        let ffmpeg_path = exe_dir.join("ffmpeg");

        if ffmpeg_path.exists() {
            return Some(FfmpegEntry::Path(ffmpeg_path));
        }
    }

    // 3. check if env variable is set (has bug)
    if let Ok(output) = Command::new("ffmpeg").arg("-version").output().await {
        if output.status.success() {
            return Some(FfmpegEntry::Env);
        }
    }

    // end. 404
    None
}

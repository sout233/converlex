use std::path::PathBuf;
use std::process::Stdio;
use futures_util::future;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;
use tokio::sync::mpsc;

#[derive(Debug, Clone)]
pub struct FfmpegCommandBuilder {
    input: Option<PathBuf>,
    output: Option<PathBuf>,
    video_bitrate: Option<u32>,
    audio_bitrate: Option<u32>,
    resolution: Option<(u32, u32)>,
    frame_rate: Option<u32>,
    sample_rate: Option<u32>,
    extra_args: Vec<String>,
}

#[allow(dead_code)]
impl FfmpegCommandBuilder {
    pub fn new() -> Self {
        Self {
            input: None,
            output: None,
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

    pub fn video_bitrate(mut self, kbps: u32) -> Self {
        self.video_bitrate = Some(kbps);
        self
    }

    pub fn audio_bitrate(mut self, kbps: u32) -> Self {
        self.audio_bitrate = Some(kbps);
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
        let input = self.input.ok_or("Missing input path")?;
        let output = self.output.ok_or("Missing output path")?;

        let mut args = vec!["-y".into(), "-i".into(), input.to_string_lossy().into_owned()];

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

        args.extend(self.extra_args);
        args.push(output.to_string_lossy().into_owned());

        Ok((output, args))
    }
}

#[derive(Debug, Clone)]
pub enum ProgressMsg {
    Progress { task_id: usize, progress: f32 },
    Done { task_id: usize },
}

/// 异步执行单个 ffmpeg 命令，并实时输出日志
pub async fn run_ffmpeg_command(id: usize, args: Vec<String>) -> anyhow::Result<()> {
    let mut child = Command::new("ffmpeg")
        .args(args)
        .stderr(Stdio::piped())
        .stdout(Stdio::null())
        .spawn()?;

    let stderr = child.stderr.take().unwrap();
    let reader = BufReader::new(stderr).lines();

    tokio::pin!(reader);

    while let Some(line) = reader.next_line().await? {
        println!("[Task {id}] {}", line);
    }

    let status = child.wait().await?;

    if !status.success() {
        eprintln!("[Task {id}] ffmpeg exited with code: {:?}", status.code());
    } else {
        println!("[Task {id}] ✅ Conversion done.");
    }

    Ok(())
}

/// 并发处理多个任务
pub async fn run_batch(
    tasks: Vec<(usize, FfmpegCommandBuilder)>,
    tx: mpsc::UnboundedSender<ProgressMsg>,
) -> anyhow::Result<()> {
    let futures = tasks.into_iter().map(|(id, builder)| {
        let tx_clone = tx.clone();

        async move {
            match builder.build() {
                Ok((_output, args)) => {
                    let callback = {
                        let tx = tx_clone.clone();
                        move |progress: f32| {
                            let _ = tx.send(ProgressMsg::Progress {
                                task_id: id,
                                progress,
                            });
                        }
                    };

                    if let Err(e) = run_ffmpeg_command_with_progress(id, args, callback).await {
                        eprintln!("[Task {id}] ❌ Error: {:?}", e);
                    } else {
                        let _ = tx_clone.send(ProgressMsg::Done { task_id: id });
                    }
                }
                Err(e) => {
                    eprintln!("[Task {id}] ❌ Invalid config: {}", e);
                }
            }
        }
    });

    future::join_all(futures).await;
    Ok(())
}

pub async fn run_ffmpeg_command_with_progress<F>(
    id: usize,
    args: Vec<String>,
    mut progress_cb: F,
) -> anyhow::Result<()>
where
    F: FnMut(f32) + Send + 'static,
{
    let mut cmd = Command::new("ffmpeg");
    cmd.args(&args)
        .stderr(Stdio::piped())  // FFmpeg 进度信息通常输出在 stderr
        .stdout(Stdio::null())
        .stdin(Stdio::null());

    let mut child = cmd.spawn()?;
    let stderr = child.stderr.take().expect("Failed to capture stderr");

    let reader = BufReader::new(stderr);
    let mut lines = reader.lines();

    let mut duration_secs: Option<f32> = None;

    while let Some(line) = lines.next_line().await? {
        if line.contains("Duration:") {
            // 提取 total 时长
            if let Some(dur) = parse_duration(&line) {
                duration_secs = Some(dur);
                println!("[Task {id}] 🎬 Duration = {}s", dur);
            }
        } else if line.contains("time=") {
            if let Some(current_time) = parse_progress_time(&line) {
                if let Some(total) = duration_secs {
                    let ratio = (current_time / total).min(1.0);
                    progress_cb(ratio);
                }
            }
        }
    }

    let status = child.wait().await?;
    if !status.success() {
        anyhow::bail!("ffmpeg exited with status {}", status);
    }

    Ok(())
}fn parse_duration(line: &str) -> Option<f32> {
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

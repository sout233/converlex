use std::path::PathBuf;
use std::process::Stdio;
use futures_util::future;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;

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
pub async fn run_batch(tasks: Vec<(usize, FfmpegCommandBuilder)>) -> anyhow::Result<()> {
    let futures = tasks.into_iter().map(|(id, builder)| async move {
        match builder.build() {
            Ok((_output, args)) => {
                if let Err(e) = run_ffmpeg_command(id, args).await {
                    eprintln!("[Task {id}] ❌ Error: {:?}", e);
                }
            }
            Err(e) => {
                eprintln!("[Task {id}] ❌ Invalid config: {}", e);
            }
        }
    });

    future::join_all(futures).await;
    Ok(())
}

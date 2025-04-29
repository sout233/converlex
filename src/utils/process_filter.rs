use std::sync::Arc;
use ez_ffmpeg::filter::frame_filter::{FrameFilter, FrameFilterContext};
use ez_ffmpeg::{AVMediaType, Frame, AVRational};

pub struct ProgressCallBacker {
    pub total_duration: i64,      // 微秒
    pub time_base: AVRational,
}

impl ProgressCallBacker {
    pub fn new() -> Self {
        Self { total_duration: 0, time_base: AVRational { num: 0, den: 0 } }
    }

    pub fn print_progress(&self, frame: &Frame) {
        if let Some(pts) = frame.pts() {
            if self.time_base.den == 0 { return } // 防错
            let time_s = pts as f64 * self.time_base.num as f64 / self.time_base.den as f64;
            let total_s = self.total_duration as f64 / 1_000_000.0;
            let pct = (time_s / total_s * 100.0).clamp(0.0, 100.0);
            println!("Progress: {:.2}% ({:.3}s / {:.3}s)", pct, time_s, total_s);
            // 这里可以改为调用你的回调： on_progress(pct as f32 / 100.0)
        }
    }
}

pub struct ProgressCallBackFilter {
    cb: Arc<ProgressCallBacker>,
}

impl ProgressCallBackFilter {
    pub fn new(cb: Arc<ProgressCallBacker>) -> Self { Self { cb } }
}

impl FrameFilter for ProgressCallBackFilter {
    fn media_type(&self) -> AVMediaType {
        AVMediaType::AVMEDIA_TYPE_AUDIO
    }
    fn filter_frame(
        &mut self,
        frame: Frame,
        _ctx: &FrameFilterContext,
    ) -> Result<Option<Frame>, String> {
        unsafe {
            if frame.as_ptr().is_null() || frame.is_empty() {
                return Ok(Some(frame));
            }
        }
        self.cb.print_progress(&frame);
        Ok(Some(frame))
    }
}

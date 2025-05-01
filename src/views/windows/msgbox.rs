use crate::utils::ffmpeg_wrapper::FfmpegEntry;

#[macro_export]
macro_rules! err_msgbox {
    // if no title
    ($msg:expr) => {{
        rfd::MessageDialog::new()
            .set_title("Error")
            .set_description($msg)
            .show();
        return;
    }};

    ($msg:expr, $title:expr) => {{
        rfd::MessageDialog::new()
            .set_title($title)
            .set_description($msg)
            .show();
        return;
    }};
}


#[macro_export]
macro_rules! unwrap_or_msgbox {
    ($opt:expr, $msg:expr) => {{
        match $opt {
            Some(v) => v,
            None => {
                rfd::MessageDialog::new()
                    .set_title("Error")
                    .set_description($msg)
                    .show();
                return;
            }
        }
    }};

    ($opt:expr) => {{
        match $opt {
            Some(v) => v,
            None => {
                rfd::MessageDialog::new()
                    .set_title("Error")
                    .set_description("发生错误，未找到所需项")
                    .show();
                return;
            }
        }
    }};
}

pub trait FxxkOption {
    fn unwrap_or_empty_string(self) -> String;
}

impl FxxkOption for Option<FfmpegEntry> {
    fn unwrap_or_empty_string(self) -> String {
        match self {
            Some(v) => v.to_string(),
            None => String::new(),
        }
    }
}

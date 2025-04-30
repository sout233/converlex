use std::fmt::Display;
use vizia::prelude::*;

use super::convertible_format::ConvertibleFormat;

#[derive(Data, Clone, Debug, PartialEq)]
pub enum MediaFormat {
    Audio(Audio),
    Video(Video),
}

impl MediaFormat {
    pub fn new(extension: &str) -> Option<Self> {
        match extension {
            "mp4" => Some(MediaFormat::Video(Video::Mp4)),
            "mkv" => Some(MediaFormat::Video(Video::Mkv)),
            "avi" => Some(MediaFormat::Video(Video::Avi)),
            "mp3" => Some(MediaFormat::Audio(Audio::Mp3)),
            "wav" => Some(MediaFormat::Audio(Audio::Wav)),
            "flac" => Some(MediaFormat::Audio(Audio::Flac)),
            _ => None,
        }
    }
}

impl Default for MediaFormat {
    fn default() -> Self {
        MediaFormat::Audio(Audio::Mp3)
    }
}

impl Display for MediaFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MediaFormat::Audio(audio) => write!(f, "{}", audio),
            MediaFormat::Video(video) => write!(f, "{}", video),
        }
    }
}

impl ConvertibleFormat for MediaFormat{
    fn get_supported_output_formats(&self) -> Vec<Box<dyn ConvertibleFormat>> {
        match self {
            MediaFormat::Audio(_) => vec![
                Box::new(MediaFormat::Audio(Audio::Mp3)),
                Box::new(MediaFormat::Audio(Audio::Wav)),
                Box::new(MediaFormat::Audio(Audio::Flac)),
                Box::new(MediaFormat::Video(Video::Mp4)),
                Box::new(MediaFormat::Video(Video::Mkv)),
                Box::new(MediaFormat::Video(Video::Avi)),
            ],
            MediaFormat::Video(_) => vec![
                Box::new(MediaFormat::Video(Video::Mp4)),
                Box::new(MediaFormat::Video(Video::Mkv)),
                Box::new(MediaFormat::Video(Video::Avi)),
                Box::new(MediaFormat::Audio(Audio::Mp3)),
                Box::new(MediaFormat::Audio(Audio::Wav)),
                Box::new(MediaFormat::Audio(Audio::Flac)),
            ],
        }
    }

    fn as_any(&self) -> &dyn ConvertibleFormat {
        match self {
            MediaFormat::Audio(_) => self,
            MediaFormat::Video(_) => self,
        }
    }
    
    fn get_ext(&self) -> String {
        match self {
            MediaFormat::Audio(audio) => audio.to_string(),
            MediaFormat::Video(video) => video.to_string(),
        }
    }
}

#[derive(Data, Clone, Debug, PartialEq)]
pub enum Video {
    Mp4,
    Mkv,
    Avi,
}

impl Display for Video {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Video::Mp4 => write!(f, "mp4"),
            Video::Mkv => write!(f, "mkv"),
            Video::Avi => write!(f, "avi"),
        }
    }
}

#[derive(Data, Clone, Debug, PartialEq)]
pub enum Audio {
    Mp3,
    Wav,
    Flac,
}

impl Display for Audio {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Audio::Mp3 => write!(f, "mp3"),
            Audio::Wav => write!(f, "wav"),
            Audio::Flac => write!(f, "flac"),
        }
    }
}

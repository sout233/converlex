use std::fmt::Display;
use vizia::prelude::*;

#[derive(Data, Clone, Debug, PartialEq)]
pub enum MediaFormat {
    Audio(Audio),
    Video(Video),
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
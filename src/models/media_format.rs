use std::fmt::{self, Display};
use strum::IntoEnumIterator;
use strum_macros::AsRefStr;
use strum_macros::EnumIter;
use vizia::prelude::*;

use crate::def_formats;

use super::convertible_format::ConvertibleFormat;
use super::convertible_format::FormatType;

#[derive(Data, Clone, Debug, PartialEq)]
pub enum MediaFormat {
    Audio(Audio),
    Video(Video),
}

impl MediaFormat {
    pub fn new(extension: &str) -> Option<Self> {
        let ext = extension.trim_start_matches('.').to_lowercase();

        if let Some(video) = Video::from_extension(&ext) {
            return Some(MediaFormat::Video(video));
        }

        if let Some(audio) = Audio::from_extension(&ext) {
            return Some(MediaFormat::Audio(audio));
        }

        None
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

impl ConvertibleFormat for MediaFormat {
    fn get_supported_output_formats(&self) -> Vec<Box<dyn ConvertibleFormat>> {
        let audio_all: Vec<Box<dyn ConvertibleFormat>> = Audio::all()
            .into_iter()
            .map(|fmt| Box::new(MediaFormat::Audio(fmt)) as Box<dyn ConvertibleFormat>)
            .collect();

        let video_all: Vec<Box<dyn ConvertibleFormat>> = Video::all()
            .into_iter()
            .map(|fmt| Box::new(MediaFormat::Video(fmt)) as Box<dyn ConvertibleFormat>)
            .collect();

        let all = audio_all
            .into_iter()
            .chain(video_all.into_iter())
            .collect::<Vec<_>>();

        match self {
            MediaFormat::Audio(_) => all,
            MediaFormat::Video(_) => all,
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
    
    fn get_decs(&self) -> Option<String> {
        match self{
            MediaFormat::Audio(audio) => audio.desc().map(|s| s.to_string()),
            MediaFormat::Video(video) => video.desc().map(|s| s.to_string()),
        }
    }
    
    fn get_format_type(&self)->FormatType {
        match self {
            MediaFormat::Audio(audio) => FormatType::Audio(audio.clone()),
            MediaFormat::Video(video) => FormatType::Video(video.clone()),
        }
    }
}

def_formats! {Video{
    Mp4(decs = "MPEG-4 Part 14, widely supported video container"),
    Mkv(decs = "Matroska Multimedia Container"),
    Avi(decs = "Audio Video Interleave, Microsoft format"),
    Mov(decs = "Apple QuickTime Movie"),
    Wmv(decs = "Windows Media Video"),
    Flv(decs = "Flash Video Format"),
    Webm(decs = "Web-optimized Matroska variant by Google"),
    Mpegts(ext = "ts")(decs = "MPEG Transport Stream"),
    Mpeg(decs = "MPEG-1 or MPEG-2 Video"),
    Mpg(decs = "Alternative extension for MPEG video"),
    Ogv(decs = "Ogg Video, Theora encoded"),
    Gif(decs = "Graphics Interchange Format, supports animation"),

    ThreeG2(ext = "3g2")(decs = "3GPP2 multimedia format"),
    ThreeGp(ext = "3gp")(decs = "3GPP multimedia format"),
    F4v(decs = "Flash MP4 video format"),
    Nut(decs = "Experimental multimedia container"),
    Psp(decs = "PlayStation Portable media format"),
    RealMedia(ext = "rm")(decs = "RealNetworks streaming format"),
    Swf(decs = "Small Web Format for vector animation"),
    Vcd(ext = "dat")(decs = "Video CD format"),
    Hds(ext = "f4m")(decs = "HTTP Dynamic Streaming, Adobe format"),
    Ismv(decs = "Smooth Streaming format from Microsoft")
}}


impl Video {
    pub fn all() -> Vec<Video> {
        Video::iter().collect()
    }

    pub fn from_extension(ext: &str) -> Option<Self> {
        let ext = ext.trim_start_matches('.').to_lowercase();
        Video::iter().find(|variant| variant.ext() == ext)
    }
}

impl fmt::Display for Video {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.ext())
    }
}

def_formats! {Audio{
    Mp3(decs = "MPEG-1 Audio Layer 3"),
    Wav(decs = "Waveform Audio File Format"),
    Flac(decs = "Free Lossless Audio Codec"),
    Aac(decs = "Advanced Audio Coding"),
    Ac3(decs = "Audio Codec 3"),
    Opus(decs = "Opus Interactive Audio Codec"),
    Vorbis(ext = "ogg")(decs = "Xiph Vorbis audio"),
    Alac(ext = "m4a")(decs = "Apple Lossless Audio Codec"),
    Amr(decs = "Adaptive Multi-Rate Audio Codec"),
    Wma(decs = "Windows Media Audio"),
    Dts(decs = "DTS Coherent Acoustics"),
    Lpcm(decs = "Linear PCM"),
    Eac3(decs = "Enhanced AC-3"),
    Dsd(ext = "dsf")(decs = "Direct Stream Digital"),
    Tta(decs = "True Audio"),
    Wv(decs = "WavPack")
}}

impl Audio {
    pub fn all() -> Vec<Audio> {
        Audio::iter().collect()
    }

    pub fn from_extension(ext: &str) -> Option<Self> {
        let ext = ext.trim_start_matches('.').to_lowercase();
        Audio::iter().find(|variant| variant.ext() == ext)
    }
}

impl Display for Audio {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.ext())
    }
}

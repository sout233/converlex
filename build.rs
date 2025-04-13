use std::fs;

fn main() {
    fs::copy("ffmpeg.exe", "target/debug/ffmpeg.exe").unwrap();
}

use std::path::Path;

pub fn get_file_extension(file_path: &str) -> String {
    let path = Path::new(file_path);
    match path.extension() {
        Some(ext) => ext.to_string_lossy().to_string(),
        None => String::new(),
    }
}
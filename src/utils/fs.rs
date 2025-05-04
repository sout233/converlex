use std::path::{Component, Path};

pub fn get_file_extension(file_path: &str) -> String {
    let path = Path::new(file_path);
    match path.extension() {
        Some(ext) => ext.to_string_lossy().to_string(),
        None => String::new(),
    }
}


/// 缩短路径，保留首个组件和文件名，折叠中间部分。
/// 如果缩短后仍超过 max_len，则只返回文件名。
pub fn shorten_path(path: &Path, max_len: usize) -> String {
    let full_str = path.to_string_lossy();
    if full_str.len() <= max_len {
        return full_str.to_string();
    }

    let components: Vec<Component> = path.components().collect();
    let sep = std::path::MAIN_SEPARATOR;

    if components.len() <= 2 {
        return full_str.to_string();
    }

    let mut parts = Vec::new();

    for (i, comp) in components.iter().enumerate() {
        let comp_str = comp.as_os_str().to_string_lossy();
        if i == 0 {
            // 保留开头，如 "C:" 或 "/"，不添加分隔符
            parts.push(comp_str.to_string());
        } else if i == components.len() - 1 {
            // 保留最后一个（通常是文件名）
            parts.push(format!("{sep}{comp_str}"));
        } else {
            // 中间的缩写成首字母+...
            let abbreviated = comp_str
                .chars()
                .next()
                .map(|c| format!("{sep}{c}..."))
                .unwrap_or_default();
            parts.push(abbreviated);
        }
    }

    let shortened = parts.concat();

    if shortened.len() > max_len {
        // 缩完仍然太长，仅返回文件名
        path.file_name()
            .and_then(|f| f.to_str())
            .unwrap_or_default()
            .to_string()
    } else {
        shortened
    }
}
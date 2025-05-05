use std::fmt::Display;

pub trait ConvertibleFormat:Send + Sync {
    fn get_supported_output_formats(&self) -> Vec<Box<dyn ConvertibleFormat>>;

    fn as_any(&self) -> &dyn ConvertibleFormat;

    fn get_ext(&self) -> String;

    fn get_decs(&self) -> Option<String>;
}

impl Display for dyn ConvertibleFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.get_ext())
    }
}

impl std::fmt::Debug for dyn ConvertibleFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.get_ext())
    }
}
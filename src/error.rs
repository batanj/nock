use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("No battery found (BAT0 or BAT1 missing)")]
    BatteryNotFound,

    #[error("Failed to read {file} file")]
    ReadError {
        file: std::path::PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("Failed to parse value as an integer")]
    ParseIntError(#[from] std::num::ParseIntError),

    #[error("Failed to parse value as a float")]
    ParseFloatError(#[from] std::num::ParseFloatError),

    #[error("I/O error")]
    IoError(#[from] std::io::Error),

    #[error("Failed to convert to utf8")]
    Utf8Error(#[from] std::str::Utf8Error),
}

impl AppError {
    pub fn log_and_fallback<T, E>(result: Result<T, E>, context: &str, fallback: T) -> T
    where
        E: std::fmt::Display,
    {
        result.unwrap_or_else(|e| {
            let mut chars = context.chars();
            let capitalized_context = match chars.next() {
                Some(f) => f.to_uppercase().to_string() + chars.as_str(),
                None => String::new(),
            };

            eprintln!("{capitalized_context} read error: {e}");
            fallback
        })
    }
}

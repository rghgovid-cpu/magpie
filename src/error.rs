use thiserror::Error;

#[derive(Error, Debug)]
pub enum MagpieError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("TOML error: {0}")]
    Toml(#[from] toml::de::Error),

    #[error("CLI error: {0}")]
    Clap(#[from] clap::Error),

    #[error("Prompt error: {0}")]
    Prompt(#[from] inquire::InquireError),

    #[error("Skill not found: {0}")]
    SkillNotFound(String),

    #[error("Source not found: {0}")]
    SourceNotFound(String),

    #[error("Operation cancelled by user")]
    Cancelled,

    #[error("Print error: {0}")]
    Print(String),

    #[error("Agent error: {0}")]
    Agent(String),

    #[error("{0}")]
    Custom(String),
}

pub type Result<T> = std::result::Result<T, MagpieError>;

impl From<String> for MagpieError {
    fn from(s: String) -> Self {
        MagpieError::Custom(s)
    }
}

impl From<toml::ser::Error> for MagpieError {
    fn from(e: toml::ser::Error) -> Self {
        MagpieError::Custom(format!("toml serialization error: {e}"))
    }
}

impl From<zip::result::ZipError> for MagpieError {
    fn from(e: zip::result::ZipError) -> Self {
        MagpieError::Custom(format!("zip error: {e}"))
    }
}

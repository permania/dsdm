use std::io;

use log::error;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum DSDMError {
    #[error("IO Error: {0}")]
    Io(#[from] io::Error),

    #[error("Template Error: {0}")]
    TemplateError(#[from] upon::Error),

    #[error("Serde Error: {0}")]
    SerdeError(#[from] serde_yaml::Error),

    #[error("Failed to calculate home directory.")]
    HomeError,

    #[error("Directory Error: {0}")]
    DirError(String),

    #[error("Module already exists.")]
    ModuleExists,

    #[error("Module does not exist.")]
    NotExists,

    #[error("Error parsing prompt.")]
    QuestionError,

    #[error("Failed to convert key to string.")]
    InvalidKey,

    #[error("Failed to parse template value.")]
    InvalidValue,
}

use std::{io, path::StripPrefixError};

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

    #[error("Walkdir Error: {0}")]
    WalkDir(#[from] walkdir::Error),

    #[error("Error copying files: {0}")]
    FSExtra(#[from] fs_extra::error::Error),

    #[error("Strip Prefix Error: {0}")]
    StripPrefix(#[from] StripPrefixError),

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

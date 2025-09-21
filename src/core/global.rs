use std::fs;

use serde::Deserialize;
use serde_yaml::Value;

use crate::{
    cli::error::DSDMError,
    modules::generate::{GLOBAL_FILE_PATH, craft_path},
};

#[derive(Debug, Deserialize)]
pub struct Delims {
    pub open: String,
    pub close: String,
}

#[derive(Debug, Deserialize)]
pub struct Globals {
    pub delimiters: Option<Delims>,
    pub templates: Option<Value>,
}

impl Default for Delims {
    fn default() -> Self {
        Self {
            open: "!(".to_string(),
            close: ")!".to_string(),
        }
    }
}

fn read_global() -> Result<Globals, DSDMError> {
    let dsdm_dir = craft_path()?;
    let global_yaml = dsdm_dir.join(GLOBAL_FILE_PATH);

    let contents = fs::read_to_string(global_yaml)?;
    let cfg: Globals = serde_yaml::from_str(&contents)?;
    Ok(cfg)
}

pub fn global_templates() -> Result<Value, DSDMError> {
    let cfg = read_global()?;
    if let Some(tpl) = cfg.templates {
        Ok(tpl)
    } else {
        Ok(Value::Null)
    }
}

pub fn delims() -> Result<Delims, DSDMError> {
    let cfg = read_global()?;
    if let Some(dlm) = cfg.delimiters {
        Ok(dlm)
    } else {
        Ok(Delims::default())
    }
}

pub fn print_globals() -> Result<(), DSDMError> {
    let dsdm_dir = craft_path()?;
    let global_yaml = dsdm_dir.join(GLOBAL_FILE_PATH);

    let contents = fs::read_to_string(global_yaml)?;
    let cfg: Globals = serde_yaml::from_str(&contents)?;
    println!("{:#?}", cfg);
    Ok(())
}

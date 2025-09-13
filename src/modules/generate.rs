use inquire::Confirm;
use log::info;

use crate::cli::{
    arg_parser::ModuleArgs,
    error::DSDMError::{self, HomeError},
};
use std::{
    env, fs,
    path::{Path, PathBuf},
};

pub const DSDM_CONFIG_PATH: &str = ".dsdm.d";
pub const MODULE_FILE_PATH: &str = "mod.yaml";
pub const GLOBAL_FILE_PATH: &str = "global.yaml";
pub const DEFAULT_EXPORT_PATH: &str = ".config";
const MODULE_TEMPLATE: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/src/yaml/",
    "mod.yaml"
));
const GLOBAL_YAML: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/src/yaml/",
    "global.yaml"
));

pub fn module(args: ModuleArgs) -> Result<(), DSDMError> {
    info!("creating new module: {}", args.title);

    let dsdm_dir = craft_path()?;

    info!("checking for config directory");
    if !dsdm_dir.exists() {
        info!("config directory doesn't exist, creating");
        config_directory()?;
    } else {
        info!("config directory exists");
    }

    let module_dir = craft_path_module(&args)?;

    if module_exists::<&String>(&args.title, args.subdir)? {
        return Err(DSDMError::ModuleExists);
    }

    ensure_dir(&module_dir, "creating module directory")?;

    let module_file = module_dir.join(MODULE_FILE_PATH);
    fs::write(module_file, MODULE_TEMPLATE)?;

    Ok(())
}

/// TODO: add option to skip prompt
pub fn delete_module(args: ModuleArgs) -> Result<(), DSDMError> {
    info!("deleting module: {}", args.title);

    let dsdm_dir = craft_path()?;

    if !dsdm_dir.exists() {
        return Err(DSDMError::DirError(String::from(
            "Config directory doesn't exist.",
        )));
    }

    let mut module_dir = craft_path_module(&args)?;

    if !module_exists::<&String>(&args.title, args.subdir)? {
        return Err(DSDMError::NotExists);
    }

    module_dir.push(&args.title);

    let ans = Confirm::new(&format!("Delete module {}?", args.title))
        .with_default(false)
        .prompt();

    match ans {
        Ok(true) => fs::remove_dir_all(module_dir)?,
        Ok(false) => std::process::exit(0),
        Err(_) => return Err(DSDMError::QuestionError),
    }

    Ok(())
}

fn config_directory() -> Result<(), DSDMError> {
    let dsdm_dir = craft_path()?;

    if !dsdm_dir.exists() {
        ensure_dir(&dsdm_dir, "creating config directory")?;
        info!("writing globals file");
        fs::write(dsdm_dir.join(GLOBAL_FILE_PATH), GLOBAL_YAML)?;
    } else {
        info!("config directory already exists");
    }

    Ok(())
}

/// Returns the path to the (.dsdm.d) directory
pub fn craft_path() -> Result<PathBuf, DSDMError> {
    dirs_next::home_dir()
        .map(|p| p.join(DSDM_CONFIG_PATH))
        .ok_or(HomeError)
}

/// Returns the path to the module defined in `args`
pub fn craft_path_module(args: &ModuleArgs) -> Result<PathBuf, DSDMError> {
    let dsdm_dir = craft_path()?;
    let mut module_dir = dsdm_dir.clone();

    if let Some(sub) = &args.subdir {
        info!("generating subdirectory path");
        module_dir.push(Path::new(sub));
        info!("generated path: {:?}", module_dir);
    }

    Ok(module_dir.join(&args.title))
}

/// Creates a directory safely
fn ensure_dir(path: &Path, msg: &str) -> Result<(), DSDMError> {
    info!("{}", msg);
    fs::create_dir_all(path)
        .map_err(|e| DSDMError::DirError(format!("Failed to create {}: {}", path.display(), e)))?;
    Ok(())
}

/// Check if a module exists
pub fn module_exists<T>(title: T, subdir: Option<String>) -> Result<bool, DSDMError>
where
    T: AsRef<Path>,
{
    let dsdm_dir = craft_path()?;

    let mut module_dir = dsdm_dir;

    if let Some(sub) = &subdir {
        module_dir.push(Path::new(sub));
    }

    module_dir.push(title);

    let module_file = module_dir.join(MODULE_FILE_PATH);
    Ok(module_dir.exists() && module_file.exists())
}

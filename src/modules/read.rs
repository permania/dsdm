use std::{fs, path::PathBuf};

use log::info;
use serde::Deserialize;
use serde_yaml::Value;

use crate::{
    cli::{arg_parser::ModuleArgs, error::DSDMError},
    core::{
        global,
        template::{build_context, render_template_file},
    },
    modules::generate::{craft_path_module, module_exists, DEFAULT_EXPORT_PATH, MODULE_FILE_PATH},
};

#[derive(Debug, Deserialize)]
struct ExportEntry {
    source: String,
    target: String,
}

#[derive(Debug, Deserialize)]
struct IncludeEntry {
    path: Option<String>,
    module: String,
}

#[derive(Debug, Deserialize)]
pub struct Module {
    exports: Option<Vec<ExportEntry>>,
    include: Option<Vec<IncludeEntry>>,
    templates: Option<Value>,
}

pub fn debug(args: ModuleArgs) -> Result<(), DSDMError> {
    info!("debugging module {}", &args.title);
    let cfg: Module = read(args)?;

    println!("{:#?}", cfg);

    Ok(())
}

/// Parse a `mod.yaml` file into a `Module` containing the instructions to apply it.
pub fn read(args: ModuleArgs) -> Result<Module, DSDMError> {
    info!("reading module {}", &args.title);
    let module_dir = craft_path_module(&args)?;

    if module_exists::<&String>(&args.title, args.subdir)? {
        let config_file = module_dir.join(MODULE_FILE_PATH);
        let contents = fs::read_to_string(config_file)?;
        Ok(serde_yaml::from_str(&contents)?)
    } else {
        Err(DSDMError::NotExists)
    }
}

pub fn apply(args: ModuleArgs) -> Result<(), DSDMError> {
    info!("applying module {}", &args.title);
    let module_dir = craft_path_module(&args)?;

    let cfg: Module = read(args.clone())?;

    let tpl = build_context(
        cfg.templates.as_ref().map(|t| t.clone()),
        global::global_templates(),
    )?;
    println!("{:#?}", tpl);

    for entry in fs::read_dir(module_dir)? {
        let file_content: String;

        let entry = entry?;
        let path = entry.path();
        if path.file_name().and_then(|n| n.to_str()) == Some(MODULE_FILE_PATH) {
            continue;
        }

        file_content = render_template_file(path, &tpl)?;

        info!("generating export path");
        let out_path = expand_tilde(get_export_path(
            args.clone(),
            &cfg.exports,
            entry.file_name().to_string_lossy().to_string(),
        )?);

        info!("writing contents to {:?}", out_path);

        if let Some(parent) = out_path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(out_path, file_content)?;
    }

    if let Some(imports) = cfg.include {
        info!("applying imports");
        for item in imports {
            let args: ModuleArgs = ModuleArgs {
                title: item.module,
                subdir: item.path,
            };
            info!("applying {:?}", args);
            apply(args)?;
        }
    }

    Ok(())
}

fn get_export_path(
    args: ModuleArgs,
    exports: &Option<Vec<ExportEntry>>,
    file_name: String,
) -> Result<PathBuf, DSDMError> {
    let mut out_path = dirs_next::home_dir()
        .ok_or(DSDMError::HomeError)?
        .join(DEFAULT_EXPORT_PATH)
        .join(args.title)
        .join(&file_name);
    info!("default generated");

    if let Some(exp) = exports.as_ref() {
        info!("export override, replacing");
        for item in exp {
            if item.source == file_name {
                info!("match with {} and {}", item.source, file_name);
                out_path = PathBuf::from(&item.target);
            }
        }
    }

    Ok(out_path)
}

fn expand_tilde(path: PathBuf) -> PathBuf {
    let path_str = path.to_string_lossy();
    let expanded = shellexpand::tilde(&path_str);
    PathBuf::from(expanded.as_ref())
}

use std::{
    borrow::Cow,
    fs, io,
    path::{Path, PathBuf},
};

use log::info;
use ptree::{Style, TreeItem, print_tree};
use serde::Deserialize;
use serde_yaml::Value;
use walkdir::WalkDir;

use crate::{
    cli::{arg_parser::ModuleArgs, error::DSDMError},
    core::{
        global,
        template::{build_context, render_template_file},
    },
    modules::generate::{DEFAULT_EXPORT_PATH, MODULE_FILE_PATH, craft_path_module, module_exists},
};

#[derive(Debug, Deserialize)]
struct ExportEntry {
    source: String,
    target: String,
}

#[derive(Debug, Deserialize)]
pub struct IncludeEntry {
    pub path: Option<String>,
    pub module: String,
}

#[derive(Debug, Deserialize)]
pub struct Module {
    exports: Option<Vec<ExportEntry>>,
    include: Option<Vec<IncludeEntry>>,
    templates: Option<Value>,
}

#[derive(Clone, Debug)]
struct DepGraph {
    name: String,
    children: Vec<DepGraph>,
}

impl TreeItem for DepGraph {
    type Child = Self;

    fn write_self<W: io::Write>(&self, f: &mut W, style: &Style) -> io::Result<()> {
        write!(f, "{}", style.paint(&self.name))
    }

    fn children(&'_ self) -> Cow<'_, [Self::Child]> {
        Cow::from(self.children.clone())
    }
}

pub fn debug(args: ModuleArgs) -> Result<(), DSDMError> {
    info!("debugging module {}", &args.title);
    let cfg: Module = read(args)?;
    println!("{:#?}", cfg);

    Ok(())
}

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

fn mod_to_dep(args: ModuleArgs) -> Result<DepGraph, DSDMError> {
    let title = args.title.clone();
    let module = read(args)?;

    Ok(DepGraph {
        name: title,
        children: if let Some(includes) = module.include {
            includes
                .into_iter()
                .map(|inc| mod_to_dep(ModuleArgs::from(inc)))
                .collect::<Result<Vec<_>, DSDMError>>()?
        } else {
            vec![]
        },
    })
}

pub fn print_dep_tree(args: ModuleArgs) -> Result<(), DSDMError> {
    let tree = mod_to_dep(args)?;
    print_tree(&tree)?;

    Ok(())
}

pub fn apply(args: ModuleArgs) -> Result<(), DSDMError> {
    info!("applying module {}", &args.title);

    let module_dir = craft_path_module(&args)?;
    let cfg: Module = read(args.clone())?;
    let tpl = build_context(cfg.templates.clone(), global::global_templates())?;

    for entry in WalkDir::new(&module_dir).into_iter().filter_map(Result::ok) {
        let relative_path = entry.path().strip_prefix(&module_dir)?.to_path_buf();

        let out_path = get_export_path(&args, &cfg.exports, &relative_path)?;

        if entry.file_type().is_dir() {
            fs::create_dir_all(&out_path)?;
        } else if entry.file_type().is_file() {
            if entry.file_name().to_string_lossy() == MODULE_FILE_PATH {
                continue;
            }

            let content = render_template_file(entry.path(), &tpl)?;

            if let Some(parent) = out_path.parent() {
                fs::create_dir_all(parent)?;
            }

            info!("writing to {:?}", out_path);
            fs::write(&out_path, content)?;
        }
    }

    if let Some(imports) = cfg.include {
        info!("applying imports");
        for item in imports {
            let sub_args = ModuleArgs {
                title: item.module,
                subdir: item.path,
            };
            apply(sub_args)?;
        }
    }

    Ok(())
}

fn get_export_path(
    args: &ModuleArgs,
    exports: &Option<Vec<ExportEntry>>,
    relative_path: &Path,
) -> Result<PathBuf, DSDMError> {
    let mut out_path = dirs_next::home_dir()
        .ok_or(DSDMError::HomeError)?
        .join(DEFAULT_EXPORT_PATH)
        .join(&args.title)
        .join(relative_path);

    if let Some(exp) = exports.as_ref() {
        for item in exp {
            let source_path = Path::new(&item.source);
            if relative_path == source_path || relative_path.starts_with(source_path) {
                let suffix = relative_path
                    .strip_prefix(source_path)
                    .unwrap_or_else(|_| Path::new(""));
                out_path = expand_tilde(PathBuf::from(&item.target));

                if !suffix.as_os_str().is_empty() {
                    out_path = out_path.join(suffix);
                }

                break;
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

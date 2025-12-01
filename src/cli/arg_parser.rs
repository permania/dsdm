use clap::{
    Args, Parser, Subcommand,
    builder::{Styles, styling::AnsiColor},
};

use crate::modules::read::IncludeEntry;

#[derive(Debug, Parser)]
#[command(version, about, long_about = None, styles=STYLES)]
pub struct DSDMArgs {
    /// The command to use
    #[clap(subcommand)]
    pub cmd: DSDMCommands,

    /// Toggle verbose logging mode
    #[clap(short, long)]
    pub verbose: bool,
}

#[derive(Debug, Subcommand)]
pub enum DSDMCommands {
    /// Create, destroy, and modify modules
    #[command(name = "mod")]
    Module(ModuleCommand),

    /// Print various pieces of debug information
    #[command()]
    Debug(DebugCommand),
}

#[derive(Debug, Args)]
pub struct ModuleCommand {
    #[clap(subcommand)]
    pub cmd: ModuleSubCommand,
}

#[derive(Debug, Args)]
pub struct DebugCommand {
    #[clap(subcommand)]
    pub cmd: DebugSubCommand,
}

#[derive(Debug, Subcommand)]
pub enum DebugSubCommand {
    /// Print information about a module
    Module(ModuleArgs),

    /// Print global configuration struct
    Global,
}

#[derive(Debug, Subcommand)]
pub enum ModuleSubCommand {
    /// Generate a template for a new module
    Create(ModuleArgs),

    /// Delete a module
    Destroy(ModuleArgs),

    /// Apply a module
    Apply(ModuleArgs),

    /// Print the dependency graph for a module
    Deps(ModuleArgs),
}

#[derive(Debug, Args, Clone)]
pub struct ModuleArgs {
    /// The title of the module
    #[clap()]
    pub title: String,

    /// Specify a subdirectory to place the module in
    #[clap(short, long)]
    pub subdir: Option<String>,
}

#[derive(Debug, Args)]
pub struct ModuleWrapper {
    /// The title of the module
    pub title: String,
}

impl From<IncludeEntry> for ModuleArgs {
    fn from(entry: IncludeEntry) -> Self {
        ModuleArgs {
            title: entry.module,
            subdir: entry.path,
        }
    }
}

const STYLES: Styles = Styles::styled()
    .header(AnsiColor::Yellow.on_default())
    .usage(AnsiColor::Yellow.on_default())
    .literal(AnsiColor::BrightCyan.on_default())
    .placeholder(AnsiColor::BrightWhite.on_default());

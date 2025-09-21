mod cli;
mod core;
mod modules;

use clap::Parser;
use cli::{
    arg_parser::{
        DSDMArgs,
        DSDMCommands::{Debug, Module},
        DebugSubCommand,
        ModuleSubCommand::{Apply, Create, Destroy},
    },
    error::DSDMError,
};
use env_logger::Builder;
use log::{LevelFilter, info};

fn run() -> Result<(), DSDMError> {
    let args = DSDMArgs::parse();

    let level = if args.verbose {
        LevelFilter::Info
    } else {
        LevelFilter::Error
    };
    Builder::new().filter_level(level).init();

    info!("{} {}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"));

    match args.cmd {
        Module(wrapper) => match wrapper.cmd {
            Create(args) => modules::generate::module(args)?,
            Destroy(args) => modules::generate::delete_module(args)?,
            Apply(args) => modules::read::apply(args)?,
        },
        Debug(wrapper) => match wrapper.cmd {
            DebugSubCommand::Module(args) => modules::read::debug(args)?,
            DebugSubCommand::Global => core::global::print_globals()?,
        },
    }

    Ok(())
}

fn main() {
    if let Err(err) = run() {
        eprintln!("{}", err);
        std::process::exit(1);
    }
    info!("end execution successfully");
}

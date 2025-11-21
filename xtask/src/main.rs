use clap::{Parser, Subcommand};
use config::BuildScript;
use simplelog::*;
use std::fs::read_to_string;

mod builder;
mod config;
mod utils;

#[macro_use]
extern crate log;
extern crate simplelog;

#[derive(Parser)]
#[command(version)]
struct Arg {
    config: String,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Clone)]
enum Commands {
    Build,
    Run {
        #[arg(short, long)]
        gdb: bool,
    },
    Clippy,
    Test,
}

fn main() {
    CombinedLogger::init(vec![TermLogger::new(
        LevelFilter::Info,
        Config::default(),
        TerminalMode::Mixed,
        ColorChoice::Auto,
    )])
    .unwrap();

    let args = Arg::parse();
    let config = read_to_string(args.config).unwrap();
    let config: BuildScript = toml::from_str(config.as_str()).unwrap();

    info!("Running build script '{}' ...", config.name);

    match args.command {
        Commands::Build => builder::build(&config).unwrap(),
        Commands::Run { gdb } => {
            builder::run(config, gdb).unwrap();
        }
        Commands::Clippy => builder::clippy(config).unwrap(),
        Commands::Test => builder::test().unwrap(),
    }
}

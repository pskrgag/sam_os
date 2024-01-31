#![feature(let_chains)]

use simplelog::*;
use std::env;
use std::fs::File;
use std::io::*;

#[macro_use]
extern crate log;
extern crate simplelog;

mod toml;
mod builder;
mod runner;

fn main() -> Result<()> {
    CombinedLogger::init(vec![TermLogger::new(
        LevelFilter::Info,
        Config::default(),
        TerminalMode::Mixed,
        ColorChoice::Auto,
    )])
    .unwrap();

    let args = env::args().collect::<Vec<_>>();

    if args.len() < 3 {
        error!("Usage: {} <build script> <image name>", args[0]);
        return Err(Error::new(ErrorKind::Other, "invalid args"));
    }

    let script = &args[1];
    // let image = &args[2];

    let mut file = match File::open(script) {
        Ok(f) => Ok(f),
        Err(e) => {
            error!("Failed to open file {}: {}", script, e);
            Err(e)
        }
    }?;

    let mut str = String::new();
    file.read_to_string(&mut str)?;

    let script = toml::process_toml(str.as_str())
        .map_err(|_| Error::new(ErrorKind::InvalidData, "Failed to parse"))?;

    info!("Building '{}' ...", script.name);

    builder::build(script)
        .map_err(|_| Error::new(ErrorKind::InvalidData, "Failed to build"))?;

    info!("Done!");
    Ok(())
}

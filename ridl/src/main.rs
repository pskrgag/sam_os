#![feature(let_chains)]

use std::io::*;

#[macro_use]
extern crate log;
extern crate simplelog;

mod frontend;
mod ir;

use simplelog::*;
use frontend::lexer::*;
use frontend::parser::*;

fn main() -> Result<()> {
    CombinedLogger::init(vec![TermLogger::new(
        LevelFilter::Debug,
        Config::default(),
        TerminalMode::Mixed,
        ColorChoice::Auto,
    )])
    .unwrap();

    let args = std::env::args().collect::<Vec<_>>();

    if args.len() < 2 {
        error!("Usage: {} <file to compile>", args[0]);
        return Err(Error::new(ErrorKind::InvalidInput, ""));
    }

    for i in &args[1..] {
        let source = std::fs::read_to_string(i).expect("Failed to read file");
        let lexer = Lexer::new(source.as_bytes());
        let mut parser = Parser::new(lexer);

        parser.parse();

        todo!();
    }

    Ok(())
}

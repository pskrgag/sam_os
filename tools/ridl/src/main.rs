use simplelog::*;
use std::io::*;

#[macro_use]
extern crate log;
extern crate simplelog;

mod ast;
mod backend;
mod frontend;

#[macro_use]
mod error_reporter;

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

    if args.len() < 3 {
        error!("Usage: {} <client|server> <file to compile>", args[0]);
        return Err(Error::new(ErrorKind::InvalidInput, ""));
    }

    let server = match args[1].as_str() {
        "client" => false,
        "server" => true,
        _ => {
            error!("Unknown mode {}. Supported <client|server>", args[1]);
            return Err(Error::new(ErrorKind::InvalidInput, ""));
        }
    };

    for i in &args[2..] {
        let source =
            std::fs::read_to_string(i).unwrap_or_else(|_| format!("Failed to read file '{}'", i));
        let reporter = error_reporter::ErrorReporter::new(source.as_bytes());
        let lexer = Lexer::new(source.as_bytes());
        let mut parser = Parser::new(lexer, &reporter);

        let ast = parser
            .parse()
            .ok_or(Error::other("Failed to parse source file"))?;

        if !server {
            backend::client::compile_client(ast, &mut std::io::stdout());
        }
    }

    Ok(())
}

use std::fs::File;
use std::io::*;
use std::path::Path;

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

pub fn generate_client<I: AsRef<Path>, O: AsRef<Path>>(idl: I, out: O) -> Result<()> {
    let source = std::fs::read_to_string(&idl)
        .unwrap_or_else(|_| format!("Failed to read file '{:?}'", idl.as_ref()));
    let reporter = error_reporter::ErrorReporter::new(source.as_bytes());
    let lexer = Lexer::new(source.as_bytes());
    let parser = Parser::new(lexer, &reporter);

    let ast = parser
        .parse()
        .ok_or(Error::other("Failed to parse source file"))?;

    // Ensure that cargo reruns the build if it changes
    println!("cargo::rerun-if-changed={:?}", idl.as_ref());

    backend::client::compile_client(
        ast,
        &mut File::create(Path::new(&std::env::var("OUT_DIR").unwrap()).join(out))?,
    );
    Ok(())
}

pub fn generate_server<I: AsRef<Path>, O: AsRef<Path>>(idl: I, out: O) -> Result<()> {
    let source = std::fs::read_to_string(&idl)
        .unwrap_or_else(|_| format!("Failed to read file '{:?}'", idl.as_ref()));
    let reporter = error_reporter::ErrorReporter::new(source.as_bytes());
    let lexer = Lexer::new(source.as_bytes());
    let parser = Parser::new(lexer, &reporter);

    let ast = parser
        .parse()
        .ok_or(Error::other("Failed to parse source file"))?;

    // Ensure that cargo reruns the build if it changes
    println!("cargo::rerun-if-changed={:?}", idl.as_ref());

    backend::server::compile_server(
        ast,
        &mut File::create(Path::new(&std::env::var("OUT_DIR").unwrap()).join(out))?,
    );
    Ok(())
}

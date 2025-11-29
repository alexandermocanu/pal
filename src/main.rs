pub mod codegen;
pub mod parser;
pub mod spec;

use clap::Parser;
use inkwell::context::Context;

use crate::{codegen::generate_codegen_module, spec::module};

/// A list of arguments that can be passed to the palc executable.
#[derive(Parser, Debug)]
struct Args {
    /// The source file that the compiler should use as an entry point to your program.
    input: std::path::PathBuf,
}

fn main() -> Result<(), anyhow::Error> {
    let args = Args::parse();

    let file = std::fs::read_to_string(args.input)?;
    let (entry_module, _) = module("main".to_string()).parse(&file)?;

    let codegen_context = Context::create();
    let codegen_module = generate_codegen_module(&codegen_context, &entry_module)?;

    codegen_module.write_bitcode_to_path("bitcode.ll");

    Ok(())
}

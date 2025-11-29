pub mod parser;
pub mod spec;

use clap::Parser;

/// A list of arguments that can be passed to the palc executable.
#[derive(Parser, Debug)]
struct Args {
    /// The source file that the compiler should use as an entry point to your program.
    input: std::path::PathBuf,
}

fn main() -> Result<(), anyhow::Error> {
    let args = Args::parse();

    let file = std::fs::read_to_string(args.input)?;

    println!("{:?}", file);

    Ok(())
}

use std::fs::File;
use std::path::PathBuf;

use clap::Parser;

mod bytecode;
mod lex;
mod parse;
mod value;
mod vm;

#[derive(Parser)]
struct Cli {
    /// script
    script: PathBuf,
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    let file = File::open(cli.script)?;

    let proto = parse::ParseProto::load(file)?;
    vm::ExeState::new().execute(&proto)?;

    Ok(())
}

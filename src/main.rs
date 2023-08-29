mod univsrg;

use clap::{ArgAction, Parser};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Input files.
    /// Support extensions include `.osz`.
    #[arg(action = ArgAction::Append)]
    inputs: Vec<String>,

    /// Output file.
    /// Supported extensions include `.osz`.
    #[arg(short)]
    output: String,
}

fn main() {
    let args = Args::parse();
}

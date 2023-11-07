mod univsrg;

use std::path::PathBuf;

use clap::{ArgAction, Parser};

use crate::univsrg::{
    osu::types::OszPath,
    traits::{AppendToUnivsrg, ToOsu},
    types::Package,
};

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
    let mut package = Package::new();
    for path in &args.inputs {
        let path = PathBuf::from(path);
        let result = match path.extension().and_then(|it| it.to_str()) {
            Some("osz") => OszPath(path.clone()).append_to_univsrg(&mut package),
            _ => {
                println!("Unsupported input type, skip.");
                continue;
            }
        };
        if result.is_err() {
            println!("Failed to parse {}", path.to_string_lossy());
            continue;
        }
    }

    let path = PathBuf::from(&args.output);
    let result = match path.extension().and_then(|it| it.to_str()) {
        Some("osz") => package.to_osu(&path),
        _ => {
            println!("Unsupported output type, abort.");
            return;
        }
    };
    result.expect(&format!("Failed to compile {}", path.to_string_lossy()));
}

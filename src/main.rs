mod cdiff;
mod fnote;

use std::path::PathBuf;
use clap::{Parser, Subcommand, ValueHint};

#[derive(Parser, Debug)]
#[clap(author, version, about)]
struct Args {
    #[clap(subcommand)]
    cmd: SubCommand,
}

#[derive(Subcommand, Debug)]
enum SubCommand {
    /// Compare text content across two HTML files.
    Cdiff {
        #[clap(value_hint = ValueHint::FilePath)]
        source_path: PathBuf,

        #[clap(value_hint = ValueHint::FilePath)]
        current_path: PathBuf,
    },

    Footnotes {
        #[clap(value_hint = ValueHint::FilePath)]
        current_path: PathBuf,
    }
}

fn main() {
    let args = Args::parse();

    match args.cmd {
        SubCommand::Cdiff { source_path, current_path } => {
            cdiff::cdiff(source_path, current_path);
        },
        SubCommand::Footnotes { current_path } => {
            fnote::update_footnotes(current_path);
        }
    }
}



use clap::Parser;
use std::path::PathBuf;
use std::process;

#[derive(Parser)]
#[command(
    name = "fed",
    about = "Open files in their default application",
    long_about = "Open one or more files using the OS default application.\n\nExamples:\n  fed photo.jpg\n  fed report.pdf notes.txt\n  fed .",
    version
)]
struct Cli {
    /// One or more files (or directories) to open
    #[arg(required = true, value_name = "FILE")]
    files: Vec<PathBuf>,
}

fn main() {
    let cli = Cli::parse();
    let mut had_error = false;

    for file in &cli.files {
        if !file.exists() {
            eprintln!("fed: '{}': no such file or directory", file.display());
            had_error = true;
            continue;
        }

        if let Err(e) = open::that(file) {
            eprintln!("fed: could not open '{}': {}", file.display(), e);
            had_error = true;
        }
    }

    if had_error {
        process::exit(1);
    }
}

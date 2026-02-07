//! Example: Download a `UDPipe` model to a path of your choice.
//!
//! Works with any language tag and any destination path. Usage:
//!
//! ```shell
//! cargo run --example download_model -- [language] [dest_dir]
//! ```
//!
//! - `language`: UD treebank tag (e.g. `english-ewt`, `german-gsd`). Default:
//!   `english-ewt`.
//! - `dest_dir`: Directory to save the model file. Default: current directory.
//!
//! The model file is named `{language}-ud-2.5-191206.udpipe`. Use its path with
//! the `parse_text` example or `Model::load()`.

#![allow(
    clippy::print_stdout,
    clippy::print_stderr,
    reason = "examples use stdout/stderr for user output"
)]

use std::env;
use std::path::Path;

fn main() {
    let args: Vec<String> = env::args().collect();

    let language = args.get(1).map_or("english-ewt", String::as_str);
    let dest_dir = args.get(2).map_or(".", String::as_str);
    let dest_path = Path::new(dest_dir).join(udpipe_rs::model_filename(language));

    if dest_path.exists() {
        println!("Model already exists: {}", dest_path.display());
        println!(
            "Use with: cargo run --example parse_text -- {} <text>",
            dest_path.display()
        );
        return;
    }

    println!(
        "Downloading {language} model to {} ...",
        dest_path
            .parent()
            .unwrap_or_else(|| Path::new("."))
            .display()
    );

    match udpipe_rs::download_model(language, dest_dir) {
        Ok(path) => {
            println!("Successfully downloaded: {path}");
        }
        Err(e) => {
            eprintln!("Error: {e}");
            eprintln!();
            eprintln!("Available models:");
            for model in udpipe_rs::AVAILABLE_MODELS.iter().take(10) {
                eprintln!("  {model}");
            }
            eprintln!("  ... and {} more", udpipe_rs::AVAILABLE_MODELS.len() - 10);
            std::process::exit(1);
        }
    }
}

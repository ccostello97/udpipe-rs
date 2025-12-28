//! Example: Download a `UDPipe` model.
//!
//! Usage:
//!
//! ```shell
//! cargo run --example download_model
//! cargo run --example download_model -- german-gsd
//! cargo run --example download_model -- french-gsd ./models
//! ```

use std::env;
use std::path::Path;

fn main() {
    let args: Vec<String> = env::args().collect();

    let language = args.get(1).map_or("english-ewt", String::as_str);
    let dest_dir = args.get(2).map_or(".", String::as_str);

    let filename = udpipe_rs::model_filename(language);
    let dest_path = Path::new(dest_dir).join(&filename);

    if dest_path.exists() {
        println!("Model already exists: {}", dest_path.display());
        return;
    }

    println!("Downloading {language} model to {dest_dir}...");

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

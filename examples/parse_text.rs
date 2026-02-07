//! Example: Parse text with `UDPipe`.
//!
//! Works with any language model at any path. Usage:
//!
//! ```shell
//! cargo run --example parse_text -- path/to/model.udpipe
//! cargo run --example parse_text -- path/to/model.udpipe "Your custom text here."
//! ```

#![allow(
    clippy::print_stdout,
    clippy::print_stderr,
    reason = "examples use stdout/stderr for user output"
)]

use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();

    let (model_path, text) = match args.as_slice() {
        [_, path] => (
            path.as_str(),
            "The quick brown fox jumps over the lazy dog.",
        ),
        [_, path, t] => (path.as_str(), t.as_str()),
        _ => {
            eprintln!("Usage: parse_text <model_path> [text]");
            eprintln!();
            eprintln!("  model_path  Path to any .udpipe model file (any language)");
            eprintln!("  text        Optional text to parse (default: sample sentence)");
            eprintln!();
            eprintln!("Example:");
            eprintln!("  cargo run --example parse_text -- ./english-ewt-ud-2.5-191206.udpipe");
            std::process::exit(1);
        }
    };

    println!("Loading model from: {model_path}");
    let model = match udpipe_rs::Model::load(model_path) {
        Ok(m) => m,
        Err(e) => {
            eprintln!("Failed to load model: {e}");
            eprintln!();
            eprintln!("Use a path to an existing .udpipe model file.");
            eprintln!("With the 'download' feature, you can fetch one with:");
            eprintln!("  cargo run --example download_model -- english-ewt .");
            std::process::exit(1);
        }
    };

    println!("Parsing: {text}");
    println!();

    // Print header
    println!(
        "{:<4} {:<15} {:<15} {:<8} {:<20} {:<10} {:<4} {:<10}",
        "ID", "FORM", "LEMMA", "UPOS", "FEATS", "DEPREL", "HEAD", "CHILDREN"
    );
    println!("{}", "-".repeat(95));

    let mut sentence_count = 0;
    let mut word_count = 0;

    let parser = model.parser(text).expect("Failed to create parser");

    for (sentence_idx, sentence) in parser.enumerate() {
        let sentence = sentence.expect("Failed to parse sentence");

        if sentence_idx > 0 {
            println!(); // Blank line between sentences
        }

        for word in &sentence.words {
            let feats = if word.feats.len() > 18 {
                format!("{}...", &word.feats[..15])
            } else {
                word.feats.clone()
            };

            let children_str = if word.children.is_empty() {
                String::from("-")
            } else {
                word.children
                    .iter()
                    .map(ToString::to_string)
                    .collect::<Vec<_>>()
                    .join(",")
            };

            println!(
                "{:<4} {:<15} {:<15} {:<8} {:<20} {:<10} {:<4} {:<10}",
                word.id,
                word.form,
                word.lemma,
                word.upostag,
                feats,
                word.deprel,
                word.head,
                children_str
            );

            word_count += 1;
        }

        sentence_count += 1;
    }

    println!();
    println!("Total words: {word_count}");
    println!("Sentences: {sentence_count}");
}

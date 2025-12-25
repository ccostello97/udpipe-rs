//! Example: Parse text with UDPipe.
//!
//! Usage:
//!   cargo run --example parse_text
//!   cargo run --example parse_text -- "Your custom text here."
//!   cargo run --example parse_text -- "Text" path/to/model.udpipe

use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();

    let text = args
        .get(1)
        .map(|s| s.as_str())
        .unwrap_or("The quick brown fox jumps over the lazy dog.");

    let model_path = args
        .get(2)
        .map(|s| s.as_str())
        .unwrap_or("english-ewt-ud-2.5-191206.udpipe");

    println!("Loading model from: {}", model_path);
    let model = match udpipe_rs::Model::load(model_path) {
        Ok(m) => m,
        Err(e) => {
            eprintln!("Failed to load model: {}", e);
            eprintln!();
            eprintln!("Download a model first with:");
            eprintln!("  cargo run --example download_model");
            std::process::exit(1);
        }
    };

    println!("Parsing: {}", text);
    println!();

    let words = model.parse(text).expect("Failed to parse");

    // Print header
    println!(
        "{:<4} {:<15} {:<15} {:<8} {:<20} {:<10} {:<4}",
        "ID", "FORM", "LEMMA", "UPOS", "FEATS", "DEPREL", "HEAD"
    );
    println!("{}", "-".repeat(80));

    let mut current_sentence = -1;
    for word in &words {
        if word.sentence_id != current_sentence {
            if current_sentence >= 0 {
                println!(); // Blank line between sentences
            }
            current_sentence = word.sentence_id;
        }

        let feats = if word.feats.len() > 18 {
            format!("{}...", &word.feats[..15])
        } else {
            word.feats.clone()
        };

        println!(
            "{:<4} {:<15} {:<15} {:<8} {:<20} {:<10} {:<4}",
            word.id, word.form, word.lemma, word.upostag, feats, word.deprel, word.head
        );
    }

    println!();
    println!("Total words: {}", words.len());
    println!(
        "Sentences: {}",
        words.last().map(|w| w.sentence_id + 1).unwrap_or(0)
    );
}

# udpipe-rs

[![Crates.io][crates-badge]][crates-url]
[![Documentation][docs-badge]][docs-url]
[![MIT/Apache-2.0 licensed][license-badge]][license-url]
[![Build Status][actions-badge]][actions-url]

[crates-badge]: https://img.shields.io/crates/v/udpipe-rs.svg
[crates-url]: https://crates.io/crates/udpipe-rs
[docs-badge]: https://docs.rs/udpipe-rs/badge.svg
[docs-url]: https://docs.rs/udpipe-rs
[license-badge]: https://img.shields.io/crates/l/udpipe-rs.svg
[license-url]: #license
[actions-badge]: https://github.com/ccostello97/udpipe-rs/workflows/CI/badge.svg
[actions-url]: https://github.com/ccostello97/udpipe-rs/actions?query=workflow%3ACI

Rust bindings for [UDPipe](https://ufal.mff.cuni.cz/udpipe) — a trainable pipeline for tokenization, tagging, lemmatization, and dependency parsing using Universal Dependencies.

## Features

- **Full parsing pipeline**: Tokenization, POS tagging, lemmatization, and dependency parsing
- **Universal Dependencies**: Output follows the [UD annotation scheme](https://universaldependencies.org/)
- **Model download utility**: Easy download of pre-trained models for 65+ languages (optional)
- **Thread-safe**: Models can be shared across threads

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
udpipe-rs = "0.1"
```

Or install via command line:

```sh
cargo add udpipe-rs
```

## Usage

### Download and load a model

```rust
use udpipe_rs::{download_model, Model};

fn main() {
    // Download model by language (saved to current directory)
    let model_path = download_model("english-ewt", ".")
        .expect("Failed to download model");

    // Load and parse
    let model = Model::load(&model_path).expect("Failed to load model");
    let words = model.parse("The quick brown fox jumps over the lazy dog.")
        .expect("Failed to parse");

    for word in words {
        println!("{:<4} {:<10} {:<6} {:<10} {:>2} <- {}",
            word.id,
            word.form,
            word.upostag,
            word.lemma,
            word.head,
            word.deprel
        );
    }
}
```

Output:

```text
1    The        DET    the         2 <- det
2    quick      ADJ    quick       5 <- amod
3    brown      ADJ    brown       5 <- amod
4    fox        NOUN   fox         5 <- nsubj
5    jumps      VERB   jump        0 <- root
6    over       ADP    over        9 <- case
7    the        DET    the         9 <- det
8    lazy       ADJ    lazy        9 <- amod
9    dog        NOUN   dog         5 <- obl
10   .          PUNCT  .           5 <- punct
```

### Available languages

Pre-trained models are available for 65+ languages. Use `udpipe_rs::AVAILABLE_MODELS` to see the full list:

```rust
// Some examples:
// "english-ewt", "english-gum", "english-lines", "english-partut"
// "german-gsd", "german-hdt"
// "french-gsd", "french-sequoia", "french-spoken"
// "spanish-ancora", "spanish-gsd"
// "dutch-alpino", "dutch-lassysmall"
// "chinese-gsd", "japanese-gsd", "korean-gsd"
// ... and many more

for lang in udpipe_rs::AVAILABLE_MODELS {
    println!("{}", lang);
}
```

### Working with morphological features

```rust
use udpipe_rs::Model;

fn main() {
    let model = Model::load("english-ewt-ud-2.5-191206.udpipe").expect("Failed to load");
    let words = model.parse("Run quickly!").expect("Failed to parse");

    for word in &words {
        // Check for imperative mood
        if word.is_verb() && word.has_feature("Mood", "Imp") {
            println!("Found imperative: {}", word.form);
        }

        // Get specific features
        if let Some(tense) = word.get_feature("Tense") {
            println!("{} has tense: {}", word.form, tense);
        }
    }
}
```

### Working with sentence structure

```rust
use udpipe_rs::Model;

fn main() {
    let model = Model::load("english-ewt-ud-2.5-191206.udpipe").expect("Failed to load");
    let words = model.parse("Hello world. Goodbye world.").expect("Failed to parse");

    // Group words by sentence
    let mut current_sentence = -1;
    for word in &words {
        if word.sentence_id != current_sentence {
            println!("\n--- Sentence {} ---", word.sentence_id + 1);
            current_sentence = word.sentence_id;
        }
        println!("  {}: {} ({})", word.id, word.form, word.upostag);
    }
}
```

### Download from custom URL

If you need to download from a different source:

```rust
use udpipe_rs::download_model_from_url;

download_model_from_url(
    "https://example.com/custom-model.udpipe",
    "custom-model.udpipe",
).expect("Failed to download");
```

## API Reference

### `Word` struct

Each parsed word contains:

| Field         | Type     | Description                                              |
|---------------|----------|----------------------------------------------------------|
| `form`        | `String` | The surface form (actual text)                           |
| `lemma`       | `String` | The lemma (dictionary form)                              |
| `upostag`     | `String` | Universal POS tag (NOUN, VERB, ADJ, etc.)                |
| `xpostag`     | `String` | Language-specific POS tag                                |
| `feats`       | `String` | Morphological features (e.g., "Mood=Imp\|VerbForm=Fin")  |
| `deprel`      | `String` | Dependency relation (root, nsubj, obj, etc.)             |
| `misc`        | `String` | Miscellaneous annotations (e.g., "SpaceAfter=No")        |
| `id`          | `i32`    | 1-based index of this word within its sentence           |
| `head`        | `i32`    | Index of head word (0 = root of sentence)                |
| `sentence_id` | `i32`    | 0-based index of the sentence this word belongs to       |

### Helper methods on `Word`

- `has_feature(key, value)` — Check if a morphological feature is present
- `get_feature(key)` — Get the value of a morphological feature
- `is_verb()` — Returns true for VERB or AUX tags
- `is_noun()` — Returns true for NOUN or PROPN tags
- `is_adjective()` — Returns true for ADJ tag
- `is_punct()` — Returns true for PUNCT tag
- `is_root()` — Returns true if this word is the sentence root
- `space_after()` — Returns true if there's a space after this word (default)

## Examples

```sh
# Download a model
cargo run --example download_model
cargo run --example download_model -- german-gsd ./models

# Parse text
cargo run --example parse_text
cargo run --example parse_text -- "Your text here."
```

## Models

Pre-trained models for 100+ treebanks are available from the [LINDAT/CLARIAH-CZ repository](https://lindat.mff.cuni.cz/repository/xmlui/handle/11234/1-3131). The `download_model` function fetches from this repository automatically.

## Build requirements

- C++ compiler with C++11 support

The build script automatically downloads the UDPipe source code and compiles it as a static library. No external tools are required.

## License

This crate is dual-licensed under MIT OR Apache-2.0.

UDPipe itself is licensed under the [Mozilla Public License 2.0](https://www.mozilla.org/en-US/MPL/2.0/).

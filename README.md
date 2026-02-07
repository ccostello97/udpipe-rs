# udpipe-rs

[![Crates.io][crates-badge]][crates-url]
[![Downloads][downloads-badge]][crates-url]
[![Documentation][docs-badge]][docs-url]
[![Build Status][actions-badge]][actions-url]
[![Coverage][codecov-badge]][codecov-url]
[![MSRV][msrv-badge]][msrv-url]
[![Dependencies][deps-badge]][deps-url]
[![MIT/Apache-2.0 licensed][license-badge]][license-url]

[crates-badge]: https://img.shields.io/crates/v/udpipe-rs.svg
[crates-url]: https://crates.io/crates/udpipe-rs
[downloads-badge]: https://img.shields.io/crates/d/udpipe-rs.svg
[docs-badge]: https://img.shields.io/docsrs/udpipe-rs
[docs-url]: https://docs.rs/udpipe-rs
[actions-badge]: https://github.com/ccostello97/udpipe-rs/workflows/CI/badge.svg
[actions-url]: https://github.com/ccostello97/udpipe-rs/actions?query=workflow%3ACI
[codecov-badge]: https://codecov.io/gh/ccostello97/udpipe-rs/graph/badge.svg
[codecov-url]: https://codecov.io/gh/ccostello97/udpipe-rs
[msrv-badge]: https://img.shields.io/badge/MSRV-1.85-blue.svg
[msrv-url]: https://blog.rust-lang.org/2025/02/20/Rust-1.85.0.html
[deps-badge]: https://deps.rs/repo/github/ccostello97/udpipe-rs/status.svg
[deps-url]: https://deps.rs/repo/github/ccostello97/udpipe-rs
[license-badge]: https://img.shields.io/crates/l/udpipe-rs.svg
[license-url]: #license

Rust bindings for [UDPipe](https://ufal.mff.cuni.cz/udpipe) — a trainable pipeline for tokenization, tagging, lemmatization, and dependency parsing using Universal Dependencies.

## Features

- **Full parsing pipeline**: Tokenization, POS tagging, lemmatization, and dependency parsing
- **Universal Dependencies**: Output follows the [UD annotation scheme](https://universaldependencies.org/)
- **Model download utility**: Easy download of pre-trained models for 65+ languages (optional)
- **Thread-friendly**: Models are `Send` (can be moved between threads)

## Installation

### Without the download feature (default)

Use this if you already have `.udpipe` model files and only need parsing. No extra dependencies.

Add to your `Cargo.toml`:

```toml
[dependencies]
udpipe-rs = "0.1"
```

Or via command line:

```sh
cargo add udpipe-rs
```

### With the download feature

Use this if you want to fetch pre-trained models by language tag (e.g. `download_model("english-ewt", ".")`) or from a custom URL. This enables the `download` feature and adds the `ureq` dependency.

Add to your `Cargo.toml`:

```toml
[dependencies]
udpipe-rs = { version = "0.1", features = ["download"] }
```

Or via command line:

```sh
cargo add udpipe-rs --features download
```

## Usage

### Load a model and parse text

Create a parser with [`Model::parser`] for a given text; it returns an iterator over **sentences**. Each sentence contains words and optional multiword tokens and comments. Use any `.udpipe` model file at any path.

```rust
use udpipe_rs::Model;

fn main() -> Result<(), udpipe_rs::UdpipeError> {
    // Load a model from a path (any language, any location)
    let model = Model::load("path/to/model.udpipe")?;

    // Create a parser for the text
    let parser = model.parser("The quick brown fox jumps over the lazy dog.")?;

    // Iterate over sentences (each item is Result<Sentence, UdpipeError>)
    for sentence in parser {
        let sentence = sentence?;
        for word in &sentence.words {
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
    Ok(())
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

To fetch a model by language tag instead of using a local file, enable the `download` feature and use [`download_model`]:

```rust,no_run
use udpipe_rs::{download_model, Model};

// Requires: udpipe-rs with feature "download"
let model_path = download_model("english-ewt", ".")?;
let model = Model::load(&model_path)?;
```

### Available languages

Pre-trained models are available for 65+ languages. Use [`udpipe_rs::AVAILABLE_MODELS`] to see the full list:

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

[`Word::feats`] is a string of pipe-separated key-value pairs (e.g. `"VerbForm=Fin|Mood=Ind|Tense=Pres"`). Parse it as needed:

```rust
use udpipe_rs::Model;

fn main() -> Result<(), udpipe_rs::UdpipeError> {
    let model = Model::load("path/to/your-model.udpipe")?;
    let parser = model.parser("Run quickly!")?;

    for sentence in parser {
        let sentence = sentence?;
        for word in &sentence.words {
            // Check for imperative: look for Mood=Imp in feats
            if word.upostag == "VERB" || word.upostag == "AUX" {
                if word.feats.contains("Mood=Imp") {
                    println!("Found imperative: {}", word.form);
                }
            }
            // Parse feats for specific keys (e.g. Tense=Pres)
            for pair in word.feats.split('|') {
                if let Some((k, v)) = pair.split_once('=') {
                    if k.trim() == "Tense" {
                        println!("{} has tense: {}", word.form, v.trim());
                    }
                }
            }
        }
    }
    Ok(())
}
```

### Working with sentence structure

The parser yields one [`Sentence`] per call to [`Iterator::next`]. Each sentence has [`words`](Sentence::words), [`multiword_tokens`](Sentence::multiword_tokens), and [`comments`](Sentence::comments):

```rust
use udpipe_rs::Model;

fn main() -> Result<(), udpipe_rs::UdpipeError> {
    let model = Model::load("english-ewt-ud-2.5-191206.udpipe")?;
    let parser = model.parser("Hello world. Goodbye world.")?;

    for (idx, sentence) in parser.enumerate() {
        let sentence = sentence?;
        println!("--- Sentence {} ---", idx + 1);
        for word in &sentence.words {
            println!("  {}: {} ({})", word.id, word.form, word.upostag);
        }
    }
    Ok(())
}
```

### Download from custom URL

With the `download` feature, [`download_model_from_url`] writes the model to a file at the given path:

```rust
use udpipe_rs::download_model_from_url;

download_model_from_url(
    "https://example.com/custom-model.udpipe",
    "custom-model.udpipe",
)?;
```

## Thread Safety

`Model` is [`Send`] but **not** [`Sync`]. This means:

- **You can move** a model to another thread (ownership transfer)
- **You cannot share** `&Model` across threads simultaneously

For concurrent access, either:

**Option 1: Wrap in `Mutex`** (shared model, serialized access)

```rust
use std::sync::{Arc, Mutex};
use udpipe_rs::Model;

let model = Arc::new(Mutex::new(Model::load("model.udpipe")?));

let model_clone = Arc::clone(&model);
std::thread::spawn(move || {
    let guard = model_clone.lock().unwrap();
    let parser = guard.parser("Hello world").unwrap();
    for sentence in parser {
        let _ = sentence.unwrap();
    }
});
```

**Option 2: Separate models per thread** (parallel access, higher memory)

```rust
use udpipe_rs::Model;

std::thread::spawn(|| {
    let model = Model::load("model.udpipe").unwrap();
    let parser = model.parser("Hello world").unwrap();
    for sentence in parser {
        let _ = sentence.unwrap();
    }
});
```

## API Reference

### [`Sentence`]

A parsed sentence from the pipeline.

| Field              | Type                  | Description                                          |
|--------------------|-----------------------|------------------------------------------------------|
| `words`            | `Vec<Word>`           | Words in the sentence (1-based IDs, no virtual root) |
| `multiword_tokens` | `Vec<MultiwordToken>` | Contractions / multiword tokens (e.g. "don't")       |
| `comments`         | `Vec<String>`         | CoNLL-U comment lines                                |

### [`Word`]

Each parsed word contains:

| Field      | Type       | Description                                            |
|------------|------------|--------------------------------------------------------|
| `form`     | `String`   | The surface form (actual text)                         |
| `lemma`    | `String`   | The lemma (dictionary form)                            |
| `upostag`  | `String`   | Universal POS tag (NOUN, VERB, ADJ, etc.)              |
| `xpostag`  | `String`   | Language-specific POS tag                              |
| `feats`    | `String`   | Morphological features (e.g. "Mood=Imp\|VerbForm=Fin") |
| `deprel`   | `String`   | Dependency relation (root, nsubj, obj, etc.)           |
| `deps`     | `String`   | Enhanced dependencies                                  |
| `misc`     | `String`   | Miscellaneous annotations (e.g. "SpaceAfter=No")       |
| `id`       | `i32`      | 1-based index of this word within its sentence         |
| `head`     | `i32`      | Index of head word (0 = root of sentence)              |
| `children` | `Vec<i32>` | Indices of child words in the dependency tree          |

Features in `feats` are pipe-separated `Key=Value` pairs; parse them as needed (e.g. check `upostag` for VERB/AUX, or search `feats` for "Mood=Imp").

## Examples

Both examples work with any language model at any path.

```sh
# Parse text (model path required)
cargo run --example parse_text -- path/to/model.udpipe
cargo run --example parse_text -- path/to/model.udpipe "Your text here."

# Download a model (requires the 'download' feature)
cargo run --example download_model -- english-ewt .
cargo run --example download_model -- german-gsd ./models
```

## Models

Pre-trained models for 100+ treebanks are available from the [LINDAT/CLARIAH-CZ repository](https://lindat.mff.cuni.cz/repository/xmlui/handle/11234/1-3131). With the `download` feature, the [`download_model`] function fetches from this repository by language tag. Without it, use any `.udpipe` file at any path with [`Model::load`].

## Requirements

**For users:** A C++ compiler with C++11 support. The build script compiles UDPipe as a static library automatically.

**For contributors:** Just Docker. See [CONTRIBUTING.md](CONTRIBUTING.md) for details.

## License

This crate is dual-licensed under MIT OR Apache-2.0.

UDPipe itself is licensed under the [Mozilla Public License 2.0](https://www.mozilla.org/en-US/MPL/2.0/).

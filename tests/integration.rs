//! Integration tests for udpipe.
//!
//! These tests download a fresh model each run to fully test the download +
//! parse flow. Require the `download` feature.

#![cfg(feature = "download")]
#![allow(
    clippy::print_stderr,
    reason = "tests use stderr for diagnostic output"
)]

use std::sync::{Mutex, OnceLock};

const MODEL_LANGUAGE: &str = "english-ewt";

/// Shared model state: temp directory, model file path, and the model wrapped
/// in Mutex. Model is wrapped in Mutex because `UDPipe` is not thread-safe for
/// concurrent access.
static MODEL: OnceLock<(tempfile::TempDir, String, Mutex<udpipe_rs::Model>)> = OnceLock::new();

/// Initialize the shared model and return a reference to its state.
fn get_model_state() -> &'static (tempfile::TempDir, String, Mutex<udpipe_rs::Model>) {
    MODEL.get_or_init(|| {
        let temp_dir = tempfile::tempdir().expect("Failed to create temp directory");

        eprintln!("Downloading {MODEL_LANGUAGE} model for integration tests...");
        let model_path = udpipe_rs::download_model(MODEL_LANGUAGE, temp_dir.path())
            .expect("Failed to download model for integration tests");

        let model = udpipe_rs::Model::load(&model_path).expect("Failed to load model");
        (temp_dir, model_path, Mutex::new(model))
    })
}

/// Parse text with the shared model, collecting all sentences.
fn parse_sentences(text: &str) -> Result<Vec<udpipe_rs::Sentence>, udpipe_rs::UdpipeError> {
    get_model_state()
        .2
        .lock()
        .expect("Model mutex poisoned")
        .parser(text)?
        .collect()
}

/// Parse text and flatten all words from all sentences.
fn parse_words(text: &str) -> Result<Vec<udpipe_rs::Word>, udpipe_rs::UdpipeError> {
    let sentences = parse_sentences(text)?;
    Ok(sentences.into_iter().flat_map(|s| s.words).collect())
}

#[test]
fn test_parse_simple_sentence() {
    let sentences = parse_sentences("Hello world!").expect("Failed to parse");

    assert!(!sentences.is_empty());
    let words: Vec<_> = sentences.iter().flat_map(|s| &s.words).collect();
    assert!(words.iter().any(|w| w.form == "Hello"));
    assert!(words.iter().any(|w| w.form == "world"));
}

#[test]
fn test_parse_multiple_sentences() {
    let sentences = parse_sentences("The cat sat. The dog ran.").expect("Failed to parse");

    // Should have two sentences
    assert_eq!(sentences.len(), 2, "Should have two sentences");

    // Each sentence should have words
    assert!(!sentences[0].words.is_empty());
    assert!(!sentences[1].words.is_empty());
}

#[test]
fn test_word_ids_are_sequential() {
    let sentences = parse_sentences("The quick brown fox.").expect("Failed to parse");

    assert!(!sentences.is_empty(), "Should have parsed sentences");

    for sentence in &sentences {
        // Word IDs should be 1-based and sequential within each sentence
        for (i, word) in sentence.words.iter().enumerate() {
            assert_eq!(
                word.id,
                i32::try_from(i + 1).expect("Word index overflow"),
                "Word ID should be sequential and 1-based"
            );
        }
    }
}

#[test]
fn test_dependency_structure() {
    let sentences = parse_sentences("The cat sleeps.").expect("Failed to parse");
    let sentence = &sentences[0];

    // Should have exactly one root (deprel == "root")
    let roots: Vec<_> = sentence
        .words
        .iter()
        .filter(|w| w.deprel == "root")
        .collect();
    assert_eq!(roots.len(), 1, "Should have exactly one root");

    // Root should be the main verb
    let root = roots[0];
    assert!(
        root.upostag == "VERB" || root.upostag == "AUX",
        "Root should be a verb, got: {}",
        root.upostag
    );
}

#[test]
fn test_morphological_features() {
    let words = parse_words("She runs quickly.").expect("Failed to parse");

    // Find the verb "runs"
    let verb = words.iter().find(|w| w.lemma == "run");
    assert!(verb.is_some(), "Should find verb 'run'");

    let verb = verb.unwrap();
    assert!(
        verb.upostag == "VERB" || verb.upostag == "AUX",
        "Should be a verb"
    );
    // Should have features
    assert!(
        !verb.feats.is_empty(),
        "Verb should have morphological features"
    );
}

#[test]
fn test_empty_input() {
    let sentences = parse_sentences("").expect("Should handle empty input");

    assert!(
        sentences.is_empty(),
        "Empty input should produce no sentences"
    );
}

#[test]
fn test_unicode_input() {
    // Test with various Unicode characters
    let sentences = parse_sentences("Héllo wörld! 你好").expect("Should handle Unicode");
    assert!(!sentences.is_empty());
}

#[test]
fn test_misc_field_space_after() {
    let words = parse_words("Hello, world!").expect("Failed to parse");

    // Check misc field for SpaceAfter annotation
    let has_space_after_no = words.iter().any(|w| w.misc.contains("SpaceAfter=No"));
    let has_no_space_annotation = words
        .iter()
        .any(|w| w.misc.is_empty() || !w.misc.contains("SpaceAfter=No"));

    // Should have at least some words with and without the annotation
    assert!(has_space_after_no, "Should have words with SpaceAfter=No");
    assert!(
        has_no_space_annotation,
        "Should have words without SpaceAfter=No"
    );
}

#[test]
fn test_xpostag_field() {
    let words = parse_words("The cat sleeps.").expect("Failed to parse");

    assert!(!words.is_empty(), "Should have parsed words");

    // xpostag field should be accessible (may be empty string for some models)
    for word in &words {
        // Verify it's a valid string (not garbage)
        assert!(word.xpostag.is_ascii() || word.xpostag.is_empty());
    }
}

#[test]
fn test_deps_field() {
    let words = parse_words("The cat sleeps.").expect("Failed to parse");

    assert!(!words.is_empty(), "Should have parsed words");

    // deps field should be accessible (may be empty for basic UD)
    for word in &words {
        // Just verify it's accessible and doesn't panic
        let _ = &word.deps;
    }
}

#[test]
fn test_children_field() {
    let sentences = parse_sentences("The cat sleeps.").expect("Failed to parse");
    let sentence = &sentences[0];

    // Find the root (should have children)
    let root = sentence.words.iter().find(|w| w.deprel == "root");
    assert!(root.is_some(), "Should have a root");

    // The root verb "sleeps" should have "cat" as a child (subject)
    // and possibly "." as punctuation child
    let root = root.unwrap();
    assert!(
        !root.children.is_empty(),
        "Root should have children, got: {:?}",
        root.children
    );
}

#[test]
fn test_parser_with_null_byte() {
    // Extract error immediately - UdpipeError doesn't borrow from model
    let err = get_model_state()
        .2
        .lock()
        .expect("Model mutex poisoned")
        .parser("Hello\0world")
        .expect_err("parser should reject null bytes");
    assert!(err.message.contains("null byte"));
}

#[test]
fn test_load_from_memory() {
    // Use the shared model's file path (avoids duplicate downloads)
    let model_path = &get_model_state().1;

    // Read model into memory
    let model_data = std::fs::read(model_path).expect("Failed to read model file");

    // Load from memory
    let model =
        udpipe_rs::Model::load_from_memory(&model_data).expect("Failed to load from memory");

    // Verify it works
    let sentences: Vec<_> = model
        .parser("Test sentence.")
        .expect("Failed to create parser")
        .collect::<Result<Vec<_>, _>>()
        .expect("Failed to parse");
    assert!(!sentences.is_empty());
}

#[test]
fn test_model_drop() {
    // Test explicit drop to help coverage track the Drop impl
    // Use the shared model's file path (avoids duplicate downloads)
    let model_path = &get_model_state().1;

    let model = udpipe_rs::Model::load(model_path).expect("Failed to load model");
    drop(model); // Explicit drop - coverage tools sometimes miss implicit drops
}

#[test]
fn test_parser_drop() {
    // Test explicit drop of parser
    let model_path = &get_model_state().1;
    let model = udpipe_rs::Model::load(model_path).expect("Failed to load model");

    let parser = model.parser("Test.").expect("Failed to create parser");
    drop(parser); // Explicit drop
}

#[test]
fn test_word_pos_tags() {
    let words = parse_words("The quick brown fox jumps.").expect("Failed to parse");

    // Check for expected POS tags
    let has_det = words.iter().any(|w| w.upostag == "DET");
    let has_noun = words
        .iter()
        .any(|w| w.upostag == "NOUN" || w.upostag == "PROPN");
    let has_adj = words.iter().any(|w| w.upostag == "ADJ");
    let has_verb = words.iter().any(|w| w.upostag == "VERB");
    let has_punct = words.iter().any(|w| w.upostag == "PUNCT");

    assert!(has_det, "Should have determiner");
    assert!(has_noun, "Should have noun");
    assert!(has_adj, "Should have adjective");
    assert!(has_verb, "Should have verb");
    assert!(has_punct, "Should have punctuation");
}

#[test]
fn test_streaming_iterator() {
    // Collect sentences while holding the lock, then release it
    let sentences: Vec<_> = get_model_state()
        .2
        .lock()
        .expect("Model mutex poisoned")
        .parser("First sentence. Second sentence.")
        .expect("Failed to create parser")
        .collect();

    // Test that we iterated sentence by sentence
    assert_eq!(sentences.len(), 2, "Should have iterated over 2 sentences");
    for sentence in sentences {
        let sentence = sentence.expect("Failed to parse sentence");
        assert!(!sentence.words.is_empty());
    }
}

#[test]
fn test_iterator_early_termination() {
    // Test that we can stop iterating early - take only 2 sentences
    let sentences: Vec<_> = get_model_state()
        .2
        .lock()
        .expect("Model mutex poisoned")
        .parser("One. Two. Three. Four. Five.")
        .expect("Failed to create parser")
        .take(2)
        .collect();

    assert_eq!(sentences.len(), 2, "Should have stopped after 2 sentences");
    for sentence in sentences {
        sentence.expect("Failed to parse");
    }
}

/// Test multiword token extraction with Spanish model.
/// Spanish has contractions like "del" (de + el), "al" (a + el) that produce
/// multiword tokens in Universal Dependencies.
#[test]
fn test_multiword_tokens_spanish() {
    // Download Spanish model
    let temp_dir = tempfile::tempdir().expect("Failed to create temp directory");
    eprintln!("Downloading spanish-gsd model for multiword token test...");
    let model_path = udpipe_rs::download_model("spanish-gsd", temp_dir.path())
        .expect("Failed to download Spanish model");

    let model = udpipe_rs::Model::load(&model_path).expect("Failed to load Spanish model");

    // "Voy al parque" contains "al" which is a contraction of "a" + "el"
    // "del" is a contraction of "de" + "el"
    let sentences: Vec<_> = model
        .parser("Voy al parque del centro.")
        .expect("Failed to create parser")
        .collect::<Result<Vec<_>, _>>()
        .expect("Failed to parse");

    assert!(!sentences.is_empty(), "Should have at least one sentence");

    // Check for multiword tokens
    let total_mwt: usize = sentences.iter().map(|s| s.multiword_tokens.len()).sum();
    eprintln!("Found {total_mwt} multiword tokens in Spanish sentence");

    // Spanish "al" and "del" should produce multiword tokens
    // Note: Depending on the model version, this may vary
    assert!(
        total_mwt > 0,
        "Spanish sentence with 'al' and 'del' should have multiword tokens"
    );

    // Verify multiword token structure
    for sentence in &sentences {
        for mwt in &sentence.multiword_tokens {
            eprintln!(
                "MWT: form='{}', id_first={}, id_last={}",
                mwt.form, mwt.id_first, mwt.id_last
            );
            assert!(!mwt.form.is_empty(), "MWT form should not be empty");
            assert!(
                mwt.id_first <= mwt.id_last,
                "MWT id_first should be <= id_last"
            );
        }
    }
}

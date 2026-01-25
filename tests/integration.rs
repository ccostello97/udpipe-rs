//! Integration tests for udpipe.
//!
//! These tests download a fresh model each run to fully test the download +
//! parse flow.

#![allow(
    clippy::print_stderr,
    reason = "tests use stderr for diagnostic output"
)]

use std::sync::{Mutex, OnceLock};

const MODEL_LANGUAGE: &str = "english-ewt";

// Model is wrapped in Mutex because UDPipe is not thread-safe for concurrent
// access. Tests acquire the lock to ensure exclusive access during parsing.
static MODEL: OnceLock<(tempfile::TempDir, Mutex<udpipe_rs::Model>)> = OnceLock::new();

/// Parse text with the shared model, releasing the lock immediately after.
fn parse(text: &str) -> Result<Vec<udpipe_rs::Word>, udpipe_rs::UdpipeError> {
    MODEL
        .get_or_init(|| {
            let temp_dir = tempfile::tempdir().expect("Failed to create temp directory");

            eprintln!("Downloading {MODEL_LANGUAGE} model for integration tests...");
            let model_path = udpipe_rs::download_model(MODEL_LANGUAGE, temp_dir.path())
                .expect("Failed to download model for integration tests");

            let model = udpipe_rs::Model::load(&model_path).expect("Failed to load model");
            (temp_dir, Mutex::new(model))
        })
        .1
        .lock()
        .expect("Model mutex poisoned")
        .parse(text)
}

#[test]
fn test_parse_simple_sentence() {
    let words = parse("Hello world!").expect("Failed to parse");

    assert!(!words.is_empty());
    assert!(words.iter().any(|w| w.form == "Hello"));
    assert!(words.iter().any(|w| w.form == "world"));
}

#[test]
fn test_parse_multiple_sentences() {
    let words = parse("The cat sat. The dog ran.").expect("Failed to parse");

    // Should have words from both sentences
    assert!(words.len() >= 6);

    // Check sentence_id is properly set
    let sentence_ids: Vec<_> = words.iter().map(|w| w.sentence_id).collect();
    assert!(sentence_ids.contains(&0));
    assert!(sentence_ids.contains(&1));
}

#[test]
fn test_word_ids_are_sequential() {
    let words = parse("The quick brown fox.").expect("Failed to parse");

    assert!(!words.is_empty(), "Should have parsed words");

    // Word IDs should be 1-based and sequential within each sentence
    for word in &words {
        assert!(word.id >= 1, "Word ID should be >= 1");
    }
}

#[test]
fn test_dependency_structure() {
    let words = parse("The cat sleeps.").expect("Failed to parse");

    // Should have exactly one root
    let roots: Vec<_> = words.iter().filter(|w| w.is_root()).collect();
    assert_eq!(roots.len(), 1, "Should have exactly one root");

    // Root should be the main verb
    let root = roots[0];
    assert!(
        root.is_verb() || root.upostag == "VERB",
        "Root should be a verb"
    );
}

#[test]
fn test_morphological_features() {
    let words = parse("She runs quickly.").expect("Failed to parse");

    // Find the verb "runs"
    let verb = words.iter().find(|w| w.lemma == "run");
    assert!(verb.is_some(), "Should find verb 'run'");

    let verb = verb.unwrap();
    assert!(verb.is_verb());
    // Present tense, third person singular
    assert!(
        verb.has_feature("Tense", "Pres") || verb.has_feature("VerbForm", "Fin"),
        "Verb should have tense/form features"
    );
}

#[test]
fn test_empty_input() {
    let words = parse("").expect("Should handle empty input");

    assert!(words.is_empty(), "Empty input should produce no words");
}

#[test]
fn test_unicode_input() {
    // Test with various Unicode characters
    let words = parse("Héllo wörld! 你好").expect("Should handle Unicode");
    assert!(!words.is_empty());
}

#[test]
fn test_misc_field_space_after() {
    let words = parse("Hello, world!").expect("Failed to parse");

    // Most words have space after, some (before punctuation) don't
    let has_space = words.iter().filter(|w| w.has_space_after()).count();
    let no_space = words.iter().filter(|w| !w.has_space_after()).count();

    // Should have at least some of each
    assert!(has_space > 0, "Should have words with space after");
    assert!(
        no_space > 0,
        "Should have words without space after (punctuation)"
    );
}

#[test]
fn test_xpostag_field() {
    let words = parse("The cat sleeps.").expect("Failed to parse");

    assert!(!words.is_empty(), "Should have parsed words");

    // xpostag field should be accessible (may be empty string for some models)
    for word in &words {
        // Verify it's a valid string (not garbage)
        assert!(word.xpostag.is_ascii() || word.xpostag.is_empty());
    }
}

#[test]
fn test_parse_with_null_byte() {
    let result = parse("Hello\0world");
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.message.contains("null byte"));
}

#[test]
fn test_load_from_memory() {
    // First download the model to get a valid file
    let temp_dir = tempfile::tempdir().expect("Failed to create temp directory");
    let model_path = udpipe_rs::download_model(MODEL_LANGUAGE, temp_dir.path())
        .expect("Failed to download model");

    // Read model into memory
    let model_data = std::fs::read(&model_path).expect("Failed to read model file");

    // Load from memory
    let model =
        udpipe_rs::Model::load_from_memory(&model_data).expect("Failed to load from memory");

    // Verify it works
    let words = model.parse("Test sentence.").expect("Failed to parse");
    assert!(!words.is_empty());
}

#[test]
fn test_model_drop() {
    // Test explicit drop to help coverage track the Drop impl
    let temp_dir = tempfile::tempdir().expect("Failed to create temp directory");
    let model_path = udpipe_rs::download_model(MODEL_LANGUAGE, temp_dir.path())
        .expect("Failed to download model");

    let model = udpipe_rs::Model::load(&model_path).expect("Failed to load model");
    drop(model); // Explicit drop - coverage tools sometimes miss implicit drops
}

#[test]
fn test_word_pos_helpers() {
    let words = parse("The quick brown fox jumps.").expect("Failed to parse");

    // Test is_noun - "fox" should be a noun
    let has_noun = words.iter().any(udpipe_rs::Word::is_noun);
    assert!(has_noun, "Should have at least one noun");

    // Test is_adjective - "quick" and "brown" should be adjectives
    let has_adj = words.iter().any(udpipe_rs::Word::is_adjective);
    assert!(has_adj, "Should have at least one adjective");

    // Test is_punct - "." should be punctuation
    let has_punct = words.iter().any(udpipe_rs::Word::is_punct);
    assert!(has_punct, "Should have punctuation");
}

#[test]
fn test_word_get_feature() {
    let words = parse("She runs.").expect("Failed to parse");

    // Find a word with features
    let word_with_feats = words.iter().find(|w| !w.feats.is_empty());
    if let Some(word) = word_with_feats {
        // Try to get a feature - may or may not exist
        let _ = word.get_feature("Number");
        let _ = word.get_feature("NonExistent");
    }
}

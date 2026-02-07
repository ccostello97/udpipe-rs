//! Benchmarks for `UDPipe` parsing performance.
//!
//! Measures parsing throughput for short, medium, and long text inputs.

#![allow(clippy::print_stderr, reason = "benchmarks use stderr for progress")]
#![allow(
    clippy::single_call_fn,
    reason = "criterion requires single-call benchmark entry points"
)]
#![allow(
    missing_docs,
    reason = "criterion_main macro generates undocumented main"
)]

use std::hint::black_box;
use std::sync::{Mutex, MutexGuard, OnceLock};

use criterion::{Criterion, Throughput, criterion_group, criterion_main};

/// Language model to download and use for benchmarks.
const MODEL_LANGUAGE: &str = "english-ewt";

/// Cached model and temp directory (kept alive for the duration of benchmarks).
/// Model is wrapped in `Mutex` because `UDPipe` is not thread-safe.
static MODEL: OnceLock<(tempfile::TempDir, Mutex<udpipe_rs::Model>)> = OnceLock::new();

/// Returns a lock guard to the shared model, initializing it on first call.
fn get_model() -> MutexGuard<'static, udpipe_rs::Model> {
    MODEL
        .get_or_init(|| {
            let temp_dir = tempfile::tempdir().expect("Failed to create temp directory");

            eprintln!("Downloading {MODEL_LANGUAGE} model for benchmarks...");
            let model_path = udpipe_rs::download_model(MODEL_LANGUAGE, temp_dir.path())
                .expect("Failed to download model for benchmarks");

            let model = udpipe_rs::Model::load(&model_path).expect("Failed to load model");
            (temp_dir, Mutex::new(model))
        })
        .1
        .lock()
        .expect("Model mutex poisoned")
}

/// Parse text and collect all sentences.
fn parse_all(text: &str) -> Vec<udpipe_rs::Sentence> {
    get_model()
        .parser(text)
        .expect("Failed to create parser")
        .collect::<Result<Vec<_>, _>>()
        .expect("Failed to parse")
}

/// Benchmarks parsing performance on various text lengths.
fn bench_parse(c: &mut Criterion) {
    // Initialize model before benchmarking (download happens here)
    drop(get_model());

    let short_text = "The quick brown fox jumps over the lazy dog.";
    let medium_text = "The quick brown fox jumps over the lazy dog. \
        She sells seashells by the seashore. \
        Peter Piper picked a peck of pickled peppers.";
    let long_text = "Natural language processing is a subfield of linguistics, \
        computer science, and artificial intelligence concerned with the \
        interactions between computers and human language. In particular, \
        how to program computers to process and analyze large amounts of \
        natural language data. The result is a computer capable of \
        understanding the contents of documents, including the contextual \
        nuances of the language within them.";

    let mut group = c.benchmark_group("parse");

    group.throughput(Throughput::Bytes(short_text.len() as u64));
    group.bench_function("short", |b| {
        b.iter(|| parse_all(black_box(short_text)));
    });

    group.throughput(Throughput::Bytes(medium_text.len() as u64));
    group.bench_function("medium", |b| {
        b.iter(|| parse_all(black_box(medium_text)));
    });

    group.throughput(Throughput::Bytes(long_text.len() as u64));
    group.bench_function("long", |b| {
        b.iter(|| parse_all(black_box(long_text)));
    });

    group.finish();
}

criterion_group!(benches, bench_parse);
criterion_main!(benches);

//! Build script for compiling `UDPipe` C++ library.

use std::path::{Path, PathBuf};
use std::{env, fs};

fn main() {
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let src_dir = manifest_dir.join("vendor/udpipe/src");

    // Collect all UDPipe C++ source files
    let mut sources = Vec::new();
    let subdirs = [
        "model",
        "morphodita",
        "parsito",
        "sentence",
        "unilib",
        "tokenizer",
        "trainer",
        "utils",
    ];

    // Main source files
    for entry in fs::read_dir(&src_dir).expect("Failed to read UDPipe src directory") {
        let entry = entry.expect("Failed to read directory entry");
        let path = entry.path();
        if path.extension().is_some_and(|ext| ext == "cpp") {
            sources.push(path);
        }
    }

    // Subdirectory source files
    for subdir in &subdirs {
        let subdir_path = src_dir.join(subdir);
        if subdir_path.exists() {
            collect_sources_recursive(&subdir_path, &mut sources);
        }
    }

    // Build UDPipe as a static library
    let mut build = cc::Build::new();
    build
        .cpp(true)
        .opt_level(2)
        .flag_if_supported("-std=c++11")
        .flag_if_supported("-w") // Suppress warnings from UDPipe
        .include(manifest_dir.join("include"))
        .include(&src_dir)
        .include(src_dir.join("model"))
        .include(src_dir.join("morphodita"))
        .include(src_dir.join("parsito"))
        .include(src_dir.join("sentence"))
        .include(src_dir.join("unilib"))
        .include(src_dir.join("utils"))
        .include(src_dir.join("tokenizer"))
        .include(src_dir.join("trainer"))
        .define("NDEBUG", None);

    for source in &sources {
        build.file(source);
    }

    // Also compile our C wrapper
    build.file(manifest_dir.join("src/udpipe_wrapper.cpp"));

    build.compile("udpipe");

    // Link C++ standard library
    let target = env::var("TARGET").unwrap();
    if target.contains("apple") {
        println!("cargo:rustc-link-lib=c++");
    } else if target.contains("windows") && target.contains("msvc") {
        // MSVC links C++ runtime automatically
    } else {
        // Linux, BSDs, Android, and other Unix-like systems use libstdc++
        println!("cargo:rustc-link-lib=stdc++");
    }

    // Tell cargo to rerun if sources change
    println!("cargo:rerun-if-changed=src/udpipe_wrapper.cpp");
    println!("cargo:rerun-if-changed=include/udpipe_rs/udpipe_wrapper.h");
    println!("cargo:rerun-if-changed=vendor/udpipe/src");
}

/// Recursively collects C++ source files from a directory.
fn collect_sources_recursive(dir: &Path, sources: &mut Vec<PathBuf>) {
    let entries = fs::read_dir(dir)
        .unwrap_or_else(|e| panic!("Failed to read directory {}: {e}", dir.display()));

    for entry in entries {
        let entry = entry.expect("Failed to read directory entry");
        let path = entry.path();
        if path.is_dir() {
            collect_sources_recursive(&path, sources);
        } else if path.extension().is_some_and(|ext| ext == "cpp") {
            sources.push(path);
        }
    }
}

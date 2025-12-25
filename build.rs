use std::env;
use std::fs::{self, File};
use std::io::{Cursor, Read};
use std::path::{Path, PathBuf};

const UDPIPE_VERSION: &str = "v1.4.0";
const UDPIPE_URL: &str = "https://github.com/ufal/udpipe/archive/refs/tags/v1.4.0.zip";

fn main() {
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let udpipe_dir = out_dir.join("udpipe");

    // Download and extract UDPipe source if not present
    if !udpipe_dir.join("src").exists() {
        download_udpipe(&out_dir, &udpipe_dir);
    }

    let src_dir = udpipe_dir.join("src");

    // Collect all UDPipe C++ source files
    let sources = collect_sources(&src_dir);

    // Build UDPipe as a static library
    let mut build = cc::Build::new();
    build
        .cpp(true)
        .opt_level(2)
        .flag_if_supported("-std=c++11")
        .flag_if_supported("-w") // Suppress warnings from UDPipe
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
    } else if target.contains("linux") {
        println!("cargo:rustc-link-lib=stdc++");
    } else if target.contains("windows") && target.contains("msvc") {
        // MSVC links C++ runtime automatically
    } else if target.contains("windows") {
        println!("cargo:rustc-link-lib=stdc++");
    }

    // Tell cargo to rerun if wrapper sources change
    println!("cargo:rerun-if-changed=src/udpipe_wrapper.cpp");
    println!("cargo:rerun-if-changed=src/udpipe_wrapper.h");
}

fn download_udpipe(out_dir: &Path, udpipe_dir: &Path) {
    // Create output directory
    fs::create_dir_all(out_dir).expect("Failed to create output directory");

    // Download the zip file using ureq
    let response = ureq::get(UDPIPE_URL)
        .call()
        .expect("Failed to download UDPipe source");

    let mut zip_data = Vec::new();
    response
        .into_body()
        .into_reader()
        .read_to_end(&mut zip_data)
        .expect("Failed to read UDPipe zip data");

    // Extract the zip file using zip crate
    let cursor = Cursor::new(zip_data);
    let mut archive = zip::ZipArchive::new(cursor).expect("Failed to read zip archive");

    for i in 0..archive.len() {
        let mut file = archive.by_index(i).expect("Failed to read zip entry");
        let outpath = match file.enclosed_name() {
            Some(path) => out_dir.join(path),
            None => continue,
        };

        if file.is_dir() {
            fs::create_dir_all(&outpath).expect("Failed to create directory");
        } else {
            if let Some(parent) = outpath.parent() {
                fs::create_dir_all(parent).expect("Failed to create parent directory");
            }
            let mut outfile = File::create(&outpath).expect("Failed to create file");
            std::io::copy(&mut file, &mut outfile).expect("Failed to write file");
        }
    }

    // Rename extracted directory to udpipe_dir
    // GitHub extracts tags without the 'v' prefix
    let version_num = UDPIPE_VERSION.trim_start_matches('v');
    let extracted_dir = out_dir.join(format!("udpipe-{}", version_num));
    if extracted_dir.exists() {
        if udpipe_dir.exists() {
            fs::remove_dir_all(udpipe_dir).expect("Failed to remove old UDPipe directory");
        }
        fs::rename(&extracted_dir, udpipe_dir).expect("Failed to rename UDPipe directory");
    }

    // Patch empty if-body bug in v1.2.0-v1.4.0
    patch_udpipe_source(udpipe_dir);
}

fn patch_udpipe_source(udpipe_dir: &Path) {
    // Fix "if (verbose)" with commented-out body in derivator_dictionary_encoder.cpp
    let file_path = udpipe_dir.join("src/morphodita/derivator/derivator_dictionary_encoder.cpp");
    if file_path.exists() {
        let content = fs::read_to_string(&file_path).expect("Failed to read file for patching");
        let patched = content.replace("if (verbose)\n//", "if (verbose) {}\n//");
        if content != patched {
            fs::write(&file_path, patched).expect("Failed to write patched file");
        }
    }
}

fn collect_sources(dir: &Path) -> Vec<PathBuf> {
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
    for entry in fs::read_dir(dir).expect("Failed to read UDPipe src directory") {
        let entry = entry.expect("Failed to read directory entry");
        let path = entry.path();
        if path.extension().is_some_and(|ext| ext == "cpp") {
            sources.push(path);
        }
    }

    // Subdirectory source files
    for subdir in &subdirs {
        let subdir_path = dir.join(subdir);
        if subdir_path.exists() {
            collect_sources_recursive(&subdir_path, &mut sources);
        }
    }

    sources
}

fn collect_sources_recursive(dir: &Path, sources: &mut Vec<PathBuf>) {
    let entries =
        fs::read_dir(dir).unwrap_or_else(|e| panic!("Failed to read directory {:?}: {}", dir, e));

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

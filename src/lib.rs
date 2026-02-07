//! Rust bindings for `UDPipe` - Universal Dependencies Pipeline.
//!
//! `UDPipe` is a trainable pipeline for tokenization, tagging, lemmatization,
//! and dependency parsing of CoNLL-U files.
//!
//! # Example
//!
//! ```no_run
//! use udpipe_rs::Model;
//!
//! // Load a model from a path (or enable the "download" feature to fetch via download_model)
//! let model = Model::load("path/to/model.udpipe").expect("Failed to load model");
//!
//! // Parse text - returns an iterator over sentences
//! for sentence in model
//!     .parser("Hello world!")
//!     .expect("Failed to create parser")
//! {
//!     let sentence = sentence.expect("Failed to parse sentence");
//!     for word in &sentence.words {
//!         println!("{}: {} ({})", word.form, word.upostag, word.deprel);
//!     }
//! }
//! ```

use std::ffi::{CStr, CString};
use std::path::Path;

#[cfg(feature = "download")]
use std::fs::File;
#[cfg(feature = "download")]
use std::io::BufWriter;

/// Error kind for `UDPipe` operations.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum UdpipeErrorKind {
    /// Text or path contained a null byte.
    NullByteInText,
    /// Model file could not be loaded (invalid path or corrupt data).
    ModelLoadFailed,
    /// Parser could not be created (invalid arguments or tokenizer failure).
    ParserCreationFailed,
    /// Parsing failed (tokenizer/tagger/parser internal error).
    ParseError,
    /// Invalid input (e.g. unknown language for download).
    InvalidInput,
    /// Download failed (network error, empty response, or write failure).
    DownloadFailed,
}

/// Error type for `UDPipe` operations.
#[derive(Debug, Clone)]
pub struct UdpipeError {
    /// The kind of error.
    pub kind: UdpipeErrorKind,
    /// The error message (details from `UDPipe` or description).
    pub message: String,
    /// Underlying error, if any (e.g. from I/O).
    source: Option<std::sync::Arc<dyn std::error::Error + Send + Sync + 'static>>,
}

impl UdpipeError {
    /// Create a new error with the given kind and message.
    pub fn new(kind: UdpipeErrorKind, message: impl Into<String>) -> Self {
        Self {
            kind,
            message: message.into(),
            source: None,
        }
    }
}

impl std::fmt::Display for UdpipeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "UDPipe error: {}", self.message)
    }
}

impl std::error::Error for UdpipeError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        self.source.as_ref().map(|e| {
            let r: &(dyn std::error::Error + 'static) = e.as_ref();
            r
        })
    }
}

impl From<std::io::Error> for UdpipeError {
    fn from(err: std::io::Error) -> Self {
        Self {
            kind: UdpipeErrorKind::ModelLoadFailed,
            message: err.to_string(),
            source: Some(std::sync::Arc::new(err)),
        }
    }
}

/// A parsed word from `UDPipe` with Universal Dependencies annotations.
///
/// Note: The virtual root word (index 0 in `UDPipe`'s internal representation)
/// is excluded from results. Word IDs are 1-based as per CoNLL-U format.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Word {
    /// The surface form (actual text).
    pub form: String,
    /// The lemma (dictionary form).
    pub lemma: String,
    /// Universal POS tag (NOUN, VERB, ADJ, etc.).
    pub upostag: String,
    /// Language-specific POS tag.
    pub xpostag: String,
    /// Morphological features (e.g., "VerbForm=Inf|Mood=Imp").
    pub feats: String,
    /// Dependency relation to head (root, nsubj, obj, etc.).
    pub deprel: String,
    /// Enhanced dependencies (graph-based).
    pub deps: String,
    /// Miscellaneous annotations (e.g., "SpaceAfter=No").
    pub misc: String,
    /// 1-based index of this word within its sentence.
    pub id: i32,
    /// Index of the head word (0 = root).
    pub head: i32,
    /// Indices of child words in the dependency tree.
    pub children: Vec<i32>,
}

/// A multiword token representing contractions (e.g., "don't" -> "do" + "n't").
///
/// In CoNLL-U format, multiword tokens span a range of word IDs and have their
/// own surface form that differs from the concatenation of their component
/// words.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MultiwordToken {
    /// The surface form of the multiword token.
    pub form: String,
    /// Miscellaneous annotations.
    pub misc: String,
    /// First word ID in the token range (inclusive).
    pub id_first: i32,
    /// Last word ID in the token range (inclusive).
    pub id_last: i32,
}

/// A parsed sentence containing all CoNLL-U data.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Sentence {
    /// The words in this sentence (excluding the virtual root).
    pub words: Vec<Word>,
    /// Multiword tokens (contractions like "don't" -> "do" + "n't").
    pub multiword_tokens: Vec<MultiwordToken>,
    /// Comments from the CoNLL-U format (e.g., "# `sent_id` = ...", "# text =
    /// ...").
    pub comments: Vec<String>,
}

/// FFI declarations for the `UDPipe` C++ wrapper.
mod ffi {
    use std::os::raw::c_char;

    /// Opaque handle to a loaded `UDPipe` model.
    #[repr(C)]
    pub struct UdpipeModel {
        /// Zero-sized field to make this type opaque.
        _private: [u8; 0],
    }

    /// Opaque handle to a streaming parser.
    #[repr(C)]
    pub struct UdpipeParser {
        /// Zero-sized field to make this type opaque.
        _private: [u8; 0],
    }

    /// Opaque handle to a parsed sentence.
    #[repr(C)]
    pub struct UdpipeSentence {
        /// Zero-sized field to make this type opaque.
        _private: [u8; 0],
    }

    /// A single word from a sentence.
    #[repr(C)]
    pub struct UdpipeWord {
        /// Word form (the actual text).
        pub form: *const c_char,
        /// Lemma (base form).
        pub lemma: *const c_char,
        /// Universal POS tag.
        pub upostag: *const c_char,
        /// Language-specific POS tag.
        pub xpostag: *const c_char,
        /// Morphological features.
        pub feats: *const c_char,
        /// Dependency relation.
        pub deprel: *const c_char,
        /// Enhanced dependencies.
        pub deps: *const c_char,
        /// Miscellaneous annotations.
        pub misc: *const c_char,
        /// Array of child word IDs.
        pub children: *const i32,
        /// Word ID (1-indexed).
        pub id: i32,
        /// Head word ID (0 = root).
        pub head: i32,
        /// Number of children.
        pub children_count: i32,
    }

    /// A multiword token.
    #[repr(C)]
    pub struct UdpipeMultiwordToken {
        /// Token form (the actual text).
        pub form: *const c_char,
        /// Miscellaneous annotations.
        pub misc: *const c_char,
        /// First word ID in the range.
        pub id_first: i32,
        /// Last word ID in the range.
        pub id_last: i32,
    }

    unsafe extern "C" {
        // Model functions (on failure C sets *out_error; valid until next API call on
        // that thread)
        pub fn udpipe_model_load(
            model_path: *const c_char,
            out_error: *mut *const c_char,
        ) -> *mut UdpipeModel;
        pub fn udpipe_model_load_from_memory(
            data: *const u8,
            len: usize,
            out_error: *mut *const c_char,
        ) -> *mut UdpipeModel;
        pub fn udpipe_model_free(model: *mut UdpipeModel);

        // Parser functions
        pub fn udpipe_parser_new(
            model: *mut UdpipeModel,
            text: *const c_char,
            out_error: *mut *const c_char,
        ) -> *mut UdpipeParser;
        pub fn udpipe_parser_next(
            parser: *mut UdpipeParser,
            out_error: *mut *const c_char,
        ) -> *mut UdpipeSentence;
        pub fn udpipe_parser_has_error(parser: *mut UdpipeParser) -> bool;
        pub fn udpipe_parser_free(parser: *mut UdpipeParser);

        // Sentence - general
        pub fn udpipe_sentence_free(sentence: *mut UdpipeSentence);

        // Sentence - words
        pub fn udpipe_sentence_word_count(sentence: *mut UdpipeSentence) -> i32;
        pub fn udpipe_sentence_get_word(sentence: *mut UdpipeSentence, index: i32) -> UdpipeWord;

        // Sentence - multiword tokens
        pub fn udpipe_sentence_multiword_token_count(sentence: *mut UdpipeSentence) -> i32;
        pub fn udpipe_sentence_get_multiword_token(
            sentence: *mut UdpipeSentence,
            index: i32,
        ) -> UdpipeMultiwordToken;

        // Sentence - comments
        pub fn udpipe_sentence_comment_count(sentence: *mut UdpipeSentence) -> i32;
        pub fn udpipe_sentence_get_comment(
            sentence: *mut UdpipeSentence,
            index: i32,
        ) -> *const c_char;
    }
}

/// Copy the error message from *`out_error`. Valid until next API call on the
/// same thread; we copy immediately. Returns a fallback if the pointer is null.
fn copy_error_message(err_ptr: *const std::os::raw::c_char) -> String {
    if err_ptr.is_null() {
        return "Unknown UDPipe error".to_owned();
    }
    // SAFETY: C++ set this to a valid null-terminated string.
    unsafe { CStr::from_ptr(err_ptr) }
        .to_string_lossy()
        .into_owned()
}

/// Call a fallible FFI function that writes *`out_error` on failure; map null
/// result to Err with the error message. Returns the pointer on success.
fn ffi_try<T>(
    out_error: &mut *const std::os::raw::c_char,
    f: impl FnOnce(*mut *const std::os::raw::c_char) -> *mut T,
    kind: UdpipeErrorKind,
) -> Result<*mut T, UdpipeError> {
    *out_error = std::ptr::null();
    let result = f(std::ptr::from_mut::<*const std::os::raw::c_char>(out_error));
    if result.is_null() {
        Err(UdpipeError::new(kind, copy_error_message(*out_error)))
    } else {
        Ok(result)
    }
}

/// `UDPipe` model wrapper.
///
/// This is the main type for loading and using `UDPipe` models.
/// Models can be loaded from files or from memory.
///
/// # Thread Safety
///
/// `Model` is [`Send`] but **not** [`Sync`]. This means:
///
/// - **Safe**: Moving a model to another thread (`std::thread::spawn`)
/// - **Safe**: Using a model from one thread at a time
/// - **Unsafe**: Sharing `&Model` across threads (won't compile)
///
/// The underlying `UDPipe` C++ library mutates internal workspace caches during
/// parsing operations. While the library uses thread-safe pools for cache
/// allocation, concurrent parsing on the same model instance would race on the
/// workspace contents.
///
/// ## Concurrent Access Patterns
///
/// If you need to parse from multiple threads, you have two options:
///
/// 1. **Shared model with mutex** (lower memory, serialized parsing):
///
/// ```no_run
/// use std::sync::{Arc, Mutex};
///
/// use udpipe_rs::Model;
///
/// let model = Arc::new(Mutex::new(Model::load("model.udpipe").unwrap()));
///
/// // In each thread: hold the lock while parsing
/// let guard = model.lock().unwrap();
/// for sentence in guard.parser("text").unwrap() {
///     // ...
/// }
/// // Lock released when guard is dropped
/// ```
///
/// 2. **Separate model per thread** (higher memory, parallel parsing):
///
/// ```no_run
/// use udpipe_rs::Model;
///
/// std::thread::scope(|s| {
///     s.spawn(|| {
///         let model = Model::load("model.udpipe").unwrap();
///         for sentence in model.parser("text from thread 1").unwrap() {
///             // ...
///         }
///     });
///     s.spawn(|| {
///         let model = Model::load("model.udpipe").unwrap();
///         for sentence in model.parser("text from thread 2").unwrap() {
///             // ...
///         }
///     });
/// });
/// ```
pub struct Model {
    /// Raw pointer to the C++ model.
    inner: *mut ffi::UdpipeModel,
}

impl std::fmt::Debug for Model {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Model")
            .field("inner", &(!self.inner.is_null()))
            .finish()
    }
}

// SAFETY: Transferring ownership of a Model to another thread is safe.
//
// Verified by auditing vendor/udpipe/src:
// - No `thread_local` storage in UDPipe, MorphoDiTa, or Parsito
// - Model data is owned via unique_ptr (no shared ownership)
// - Internal caches use atomic spin-locks (threadsafe_stack with atomic_flag)
// - Global statics (ragel_map, lzma allocators) are read-only after init
// - Our C++ wrapper uses thread_local only for error messages, which are
//   captured immediately after each FFI call on the calling thread
unsafe impl Send for Model {}

impl Model {
    /// Load a model from a file path.
    ///
    /// # Errors
    ///
    /// Returns an error if the path contains a null byte or if the model cannot
    /// be loaded.
    ///
    /// # Example
    /// ```no_run
    /// use udpipe_rs::Model;
    /// let model = Model::load("english-ewt-ud-2.5-191206.udpipe").expect("Failed to load model");
    /// ```
    pub fn load(path: impl AsRef<Path>) -> Result<Self, UdpipeError> {
        let path_str = path.as_ref().to_string_lossy();
        let c_path = CString::new(path_str.as_bytes()).map_err(|_| {
            UdpipeError::new(
                UdpipeErrorKind::NullByteInText,
                "Invalid path (contains null byte)",
            )
        })?;

        let mut out_error: *const std::os::raw::c_char = std::ptr::null();
        let model = ffi_try(
            &mut out_error,
            |e| {
                // SAFETY: `c_path` is a valid NUL-terminated C string; `e` is a valid out-error
                // pointer.
                unsafe { ffi::udpipe_model_load(c_path.as_ptr(), e) }
            },
            UdpipeErrorKind::ModelLoadFailed,
        )?;
        Ok(Self { inner: model })
    }

    /// Load a model from a byte slice.
    ///
    /// This is useful for loading models from network sources or embedded data.
    ///
    /// # Errors
    ///
    /// Returns an error if the data is empty or not a valid `UDPipe` model.
    ///
    /// # Example
    /// ```no_run
    /// use udpipe_rs::Model;
    /// let model_data =
    ///     std::fs::read("english-ewt-ud-2.5-191206.udpipe").expect("Failed to read model");
    /// let model = Model::load_from_memory(&model_data).expect("Failed to load model");
    /// ```
    pub fn load_from_memory(data: &[u8]) -> Result<Self, UdpipeError> {
        let mut out_error: *const std::os::raw::c_char = std::ptr::null();
        let model = ffi_try(
            &mut out_error,
            |e| {
                // SAFETY: `data` is a valid slice; `e` is a valid out-error pointer.
                unsafe { ffi::udpipe_model_load_from_memory(data.as_ptr(), data.len(), e) }
            },
            UdpipeErrorKind::ModelLoadFailed,
        )?;
        Ok(Self { inner: model })
    }

    /// Create a parser for the given text.
    ///
    /// Returns an iterator that yields sentences one at a time. Each sentence
    /// is tokenized, tagged, lemmatized, and parsed for dependencies.
    ///
    /// # Errors
    ///
    /// Returns an error if the text contains a null byte or if the parser
    /// cannot be created.
    ///
    /// # Example
    /// ```no_run
    /// use udpipe_rs::Model;
    ///
    /// let model = Model::load("english-ewt-ud-2.5-191206.udpipe").expect("Failed to load");
    /// for sentence in model
    ///     .parser("The quick brown fox.")
    ///     .expect("Failed to create parser")
    /// {
    ///     let sentence = sentence.expect("Failed to parse sentence");
    ///     for word in &sentence.words {
    ///         println!("{} -> {} ({})", word.form, word.lemma, word.upostag);
    ///     }
    /// }
    /// ```
    pub fn parser(&self, text: &str) -> Result<Parser<'_>, UdpipeError> {
        let c_text = CString::new(text).map_err(|_| {
            UdpipeError::new(
                UdpipeErrorKind::NullByteInText,
                "Invalid text (contains null byte)",
            )
        })?;

        let mut out_error: *const std::os::raw::c_char = std::ptr::null();
        let parser = ffi_try(
            &mut out_error,
            |e| {
                // SAFETY: `self.inner` is a valid model; `c_text` is NUL-terminated; `e` is a
                // valid out-error pointer.
                unsafe { ffi::udpipe_parser_new(self.inner, c_text.as_ptr(), e) }
            },
            UdpipeErrorKind::ParserCreationFailed,
        )?;
        Ok(Parser {
            inner: parser,
            errored: false,
            _model: self,
        })
    }
}

/// Convert a C string pointer to an owned `String`.
fn ptr_to_string(ptr: *const std::os::raw::c_char) -> String {
    if ptr.is_null() {
        return String::new();
    }
    // SAFETY: FFI guarantees the pointer is valid and null-terminated.
    unsafe { CStr::from_ptr(ptr) }
        .to_string_lossy()
        .into_owned()
}

impl Drop for Model {
    fn drop(&mut self) {
        if !self.inner.is_null() {
            // SAFETY: `self.inner` is valid and we have exclusive ownership.
            unsafe { ffi::udpipe_model_free(self.inner) };
        }
    }
}

/// A streaming parser that yields sentences one at a time.
///
/// Created by [`Model::parser`]. Implements [`Iterator`] where each item is a
/// [`Result<Sentence, UdpipeError>`].
///
/// Once an error occurs, the iterator is "fused" and will return `None` for
/// all subsequent calls.
pub struct Parser<'a> {
    /// Raw pointer to the C++ parser.
    inner: *mut ffi::UdpipeParser,
    /// Whether an error has occurred (fuses the iterator).
    errored: bool,
    /// Reference to the model so it cannot be dropped while the parser exists.
    _model: &'a Model,
}

impl std::fmt::Debug for Parser<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Parser")
            .field("errored", &self.errored)
            .finish_non_exhaustive()
    }
}

// SAFETY: Parser holds a pointer to C++ state that is tied to a Model.
// Like Model, it can be sent to another thread but not shared.
unsafe impl Send for Parser<'_> {}

impl Iterator for Parser<'_> {
    type Item = Result<Sentence, UdpipeError>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.errored || self.inner.is_null() {
            return None;
        }

        let mut out_error: *const std::os::raw::c_char = std::ptr::null();
        // SAFETY: `self.inner` is a valid parser; `out_error` is a valid out-error
        // pointer.
        let sentence_ptr = unsafe { ffi::udpipe_parser_next(self.inner, &raw mut out_error) };

        if sentence_ptr.is_null() {
            // SAFETY: `self.inner` is a valid parser.
            if unsafe { ffi::udpipe_parser_has_error(self.inner) } {
                self.errored = true;
                return Some(Err(UdpipeError::new(
                    UdpipeErrorKind::ParseError,
                    copy_error_message(out_error),
                )));
            }
            return None;
        }

        let ptr = sentence_ptr;
        Some(Ok({
            // SAFETY: `ptr` is a valid, non-null pointer from `udpipe_parser_next`.
            let word_count = unsafe { ffi::udpipe_sentence_word_count(ptr) };
            let mut words = Vec::with_capacity(usize::try_from(word_count).unwrap_or(0));
            for i in 0..word_count {
                // SAFETY: ptr valid; index in range (see above).
                let w = unsafe { ffi::udpipe_sentence_get_word(ptr, i) };
                let children = if w.children.is_null() || w.children_count <= 0 {
                    Vec::new()
                } else {
                    let count = usize::try_from(w.children_count).unwrap_or(0);
                    // SAFETY: w.children is valid for w.children_count elements.
                    unsafe { std::slice::from_raw_parts(w.children, count) }.to_vec()
                };
                words.push(Word {
                    form: ptr_to_string(w.form),
                    lemma: ptr_to_string(w.lemma),
                    upostag: ptr_to_string(w.upostag),
                    xpostag: ptr_to_string(w.xpostag),
                    feats: ptr_to_string(w.feats),
                    deprel: ptr_to_string(w.deprel),
                    deps: ptr_to_string(w.deps),
                    misc: ptr_to_string(w.misc),
                    id: w.id,
                    head: w.head,
                    children,
                });
            }
            // SAFETY: ptr valid (see above).
            let mwt_count = unsafe { ffi::udpipe_sentence_multiword_token_count(ptr) };
            let mut multiword_tokens = Vec::with_capacity(usize::try_from(mwt_count).unwrap_or(0));
            for i in 0..mwt_count {
                // SAFETY: ptr valid; index in range (see above).
                let mwt = unsafe { ffi::udpipe_sentence_get_multiword_token(ptr, i) };
                multiword_tokens.push(MultiwordToken {
                    form: ptr_to_string(mwt.form),
                    misc: ptr_to_string(mwt.misc),
                    id_first: mwt.id_first,
                    id_last: mwt.id_last,
                });
            }
            // SAFETY: ptr valid (see above).
            let comment_count = unsafe { ffi::udpipe_sentence_comment_count(ptr) };
            let mut comments = Vec::with_capacity(usize::try_from(comment_count).unwrap_or(0));
            for i in 0..comment_count {
                comments.push(ptr_to_string(
                    // SAFETY: ptr valid; index in range (see above).
                    unsafe { ffi::udpipe_sentence_get_comment(ptr, i) },
                ));
            }
            // SAFETY: ptr is our exclusive ownership from udpipe_parser_next; we may free
            // it.
            unsafe { ffi::udpipe_sentence_free(ptr) };
            Sentence {
                words,
                multiword_tokens,
                comments,
            }
        }))
    }
}

impl Drop for Parser<'_> {
    fn drop(&mut self) {
        if !self.inner.is_null() {
            // SAFETY: `self.inner` is valid and we have exclusive ownership.
            unsafe { ffi::udpipe_parser_free(self.inner) };
        }
    }
}

/// Available pre-trained models from Universal Dependencies 2.5.
///
/// These models are hosted at the [LINDAT/CLARIAH-CZ repository](https://lindat.mff.cuni.cz/repository/xmlui/handle/11234/1-3131).
/// Enable the `download` feature and use [`download_model`] to fetch them.
pub const AVAILABLE_MODELS: &[&str] = &[
    "afrikaans-afribooms",
    "ancient_greek-perseus",
    "ancient_greek-proiel",
    "arabic-padt",
    "armenian-armtdp",
    "basque-bdt",
    "belarusian-hse",
    "bulgarian-btb",
    "buryat-bdt",
    "catalan-ancora",
    "chinese-gsd",
    "chinese-gsdsimp",
    "classical_chinese-kyoto",
    "coptic-scriptorium",
    "croatian-set",
    "czech-cac",
    "czech-cltt",
    "czech-fictree",
    "czech-pdt",
    "danish-ddt",
    "dutch-alpino",
    "dutch-lassysmall",
    "english-ewt",
    "english-gum",
    "english-lines",
    "english-partut",
    "estonian-edt",
    "estonian-ewt",
    "finnish-ftb",
    "finnish-tdt",
    "french-gsd",
    "french-partut",
    "french-sequoia",
    "french-spoken",
    "galician-ctg",
    "galician-treegal",
    "german-gsd",
    "german-hdt",
    "gothic-proiel",
    "greek-gdt",
    "hebrew-htb",
    "hindi-hdtb",
    "hungarian-szeged",
    "indonesian-gsd",
    "irish-idt",
    "italian-isdt",
    "italian-partut",
    "italian-postwita",
    "italian-twittiro",
    "italian-vit",
    "japanese-gsd",
    "kazakh-ktb",
    "korean-gsd",
    "korean-kaist",
    "kurmanji-mg",
    "latin-ittb",
    "latin-perseus",
    "latin-proiel",
    "latvian-lvtb",
    "lithuanian-alksnis",
    "lithuanian-hse",
    "maltese-mudt",
    "marathi-ufal",
    "north_sami-giella",
    "norwegian-bokmaal",
    "norwegian-nynorsk",
    "norwegian-nynorsklia",
    "old_church_slavonic-proiel",
    "old_french-srcmf",
    "old_russian-torot",
    "persian-seraji",
    "polish-lfg",
    "polish-pdb",
    "polish-sz",
    "portuguese-bosque",
    "portuguese-br",
    "portuguese-gsd",
    "romanian-nonstandard",
    "romanian-rrt",
    "russian-gsd",
    "russian-syntagrus",
    "russian-taiga",
    "sanskrit-ufal",
    "scottish_gaelic-arcosg",
    "serbian-set",
    "slovak-snk",
    "slovenian-ssj",
    "slovenian-sst",
    "spanish-ancora",
    "spanish-gsd",
    "swedish-lines",
    "swedish-talbanken",
    "tamil-ttb",
    "telugu-mtg",
    "turkish-imst",
    "ukrainian-iu",
    "upper_sorbian-ufal",
    "urdu-udtb",
    "uyghur-udt",
    "vietnamese-vtb",
    "wolof-wtb",
];

/// Base URL for the LINDAT/CLARIAH-CZ model repository (UD 2.5).
#[cfg(feature = "download")]
const MODEL_BASE_URL: &str =
    "https://lindat.mff.cuni.cz/repository/xmlui/bitstream/handle/11234/1-3131";

/// Download a pre-trained model by language identifier.
///
/// Requires the `download` feature. Downloads a model from the [LINDAT/CLARIAH-CZ repository](https://lindat.mff.cuni.cz/repository/xmlui/handle/11234/1-3131)
/// to the specified destination directory. Returns the path to the downloaded
/// model file.
///
/// # Arguments
///
/// * `language` - Language identifier (e.g., "english-ewt", "dutch-alpino",
///   "german-gsd"). See [`AVAILABLE_MODELS`] for the full list.
/// * `dest_dir` - Directory where the model will be saved.
///
/// # Errors
///
/// Returns an error if the language is not in [`AVAILABLE_MODELS`] or if the
/// download fails.
///
/// # Example
///
/// ```no_run
/// use udpipe_rs::{Model, download_model};
///
/// // Download English model to current directory
/// let model_path = download_model("english-ewt", ".").expect("Failed to download");
/// println!("Model saved to: {}", model_path);
///
/// // Load and use
/// let model = Model::load(&model_path).expect("Failed to load");
/// ```
#[cfg(feature = "download")]
#[cfg_attr(docsrs, doc(cfg(feature = "download")))]
pub fn download_model(language: &str, dest_dir: impl AsRef<Path>) -> Result<String, UdpipeError> {
    let dest_dir = dest_dir.as_ref();

    if !AVAILABLE_MODELS.contains(&language) {
        return Err(UdpipeError::new(
            UdpipeErrorKind::InvalidInput,
            format!(
                "Unknown language '{}'. Use one of: {}",
                language,
                AVAILABLE_MODELS[..5].join(", ") + ", ..."
            ),
        ));
    }

    let filename = model_filename(language);
    let dest_path = dest_dir.join(&filename);
    let url = format!("{MODEL_BASE_URL}/{filename}");

    download_model_from_url(&url, &dest_path)?;

    Ok(dest_path.to_string_lossy().into_owned())
}

/// Download a model from a custom URL to a local file path.
///
/// Requires the `download` feature. Use this if you need to download models from a different source or version.
/// For standard models, prefer [`download_model`].
///
/// # Errors
///
/// Returns an error if the download fails, the response is empty, or the file
/// cannot be written.
///
/// # Example
///
/// ```no_run
/// use udpipe_rs::download_model_from_url;
///
/// download_model_from_url(
///     "https://example.com/custom-model.udpipe",
///     "custom-model.udpipe",
/// )
/// .expect("Failed to download");
/// ```
#[cfg(feature = "download")]
#[cfg_attr(docsrs, doc(cfg(feature = "download")))]
pub fn download_model_from_url(url: &str, path: impl AsRef<Path>) -> Result<(), UdpipeError> {
    let path = path.as_ref();

    let response = ureq::get(url).call().map_err(|e| {
        UdpipeError::new(
            UdpipeErrorKind::DownloadFailed,
            format!("Failed to download: {e}"),
        )
    })?;

    let file = File::create(path)?;
    let mut writer = BufWriter::new(file);
    let bytes_written = std::io::copy(&mut response.into_body().into_reader(), &mut writer)?;

    if bytes_written == 0 {
        return Err(UdpipeError::new(
            UdpipeErrorKind::DownloadFailed,
            "Downloaded file is empty",
        ));
    }

    Ok(())
}

/// Returns the expected filename for a given language model.
///
/// # Example
///
/// ```
/// assert_eq!(
///     udpipe_rs::model_filename("english-ewt"),
///     "english-ewt-ud-2.5-191206.udpipe"
/// );
/// ```
#[must_use]
pub fn model_filename(language: &str) -> String {
    format!("{language}-ud-2.5-191206.udpipe")
}

#[cfg(test)]
mod tests {
    use std::error::Error;

    use super::*;

    #[test]
    fn test_model_filename() {
        assert_eq!(
            model_filename("english-ewt"),
            "english-ewt-ud-2.5-191206.udpipe"
        );
        assert_eq!(
            model_filename("dutch-alpino"),
            "dutch-alpino-ud-2.5-191206.udpipe"
        );
    }

    #[test]
    fn test_available_models_contains_common_languages() {
        assert!(AVAILABLE_MODELS.contains(&"english-ewt"));
        assert!(AVAILABLE_MODELS.contains(&"german-gsd"));
        assert!(AVAILABLE_MODELS.contains(&"french-gsd"));
        assert!(AVAILABLE_MODELS.contains(&"spanish-ancora"));
    }

    #[test]
    fn test_available_models_sorted() {
        let mut sorted = AVAILABLE_MODELS.to_vec();
        sorted.sort_unstable();
        assert_eq!(AVAILABLE_MODELS, sorted.as_slice());
    }

    #[test]
    #[cfg(feature = "download")]
    fn test_download_model_invalid_language() {
        let result = download_model("invalid-language-xyz", ".");
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err.kind, UdpipeErrorKind::InvalidInput);
        assert!(err.message.contains("Unknown language"));
    }

    #[test]
    fn test_udpipe_error_display() {
        let err = UdpipeError::new(UdpipeErrorKind::ParseError, "test error");
        assert_eq!(format!("{err}"), "UDPipe error: test error");
    }

    #[test]
    fn test_udpipe_error_from_io() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let err: UdpipeError = io_err.into();
        assert!(err.message.contains("not found"));
        assert!(err.source().is_some());
    }

    #[test]
    #[cfg_attr(miri, ignore)]
    fn test_model_load_nonexistent_file() {
        let result = Model::load("/nonexistent/path/to/model.udpipe");
        assert!(result.is_err());
    }

    #[test]
    #[cfg_attr(miri, ignore)]
    fn test_model_load_path_with_null_byte() {
        let result = Model::load("path\0with\0nulls.udpipe");
        let err = result.expect_err("expected error");
        assert!(err.message.contains("null byte"));
    }

    #[test]
    #[cfg_attr(miri, ignore)]
    fn test_model_load_from_memory_empty() {
        let result = Model::load_from_memory(&[]);
        assert!(result.is_err());
    }

    #[test]
    #[cfg_attr(miri, ignore)]
    fn test_model_load_from_memory_invalid() {
        let garbage = b"this is not a valid udpipe model";
        let result = Model::load_from_memory(garbage);
        assert!(result.is_err());
    }

    #[test]
    #[cfg_attr(miri, ignore)]
    fn test_parser_with_null_model() {
        let model = Model {
            inner: std::ptr::null_mut(),
        };
        let result = model.parser("test");
        let err = result.unwrap_err();
        assert!(err.message.contains("Invalid arguments"));
    }

    #[test]
    fn test_model_debug() {
        let model = Model {
            inner: std::ptr::null_mut(),
        };
        let debug_str = format!("{model:?}");
        assert!(debug_str.contains("Model"));
        assert!(debug_str.contains("inner"));
    }

    #[test]
    fn test_parser_debug() {
        let model = Model {
            inner: std::ptr::null_mut(),
        };
        let parser = Parser {
            inner: std::ptr::null_mut(),
            errored: false,
            _model: &model,
        };
        let debug_str = format!("{parser:?}");
        assert!(debug_str.contains("Parser"));
        assert!(debug_str.contains("errored"));
    }

    #[test]
    #[cfg(feature = "download")]
    #[cfg_attr(miri, ignore)]
    fn test_download_model_from_url_invalid_url() {
        let temp_dir = tempfile::tempdir().unwrap();
        let path = temp_dir.path().join("model.udpipe");
        let result = download_model_from_url("http://invalid.invalid/no-such-model", &path);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err.kind, UdpipeErrorKind::DownloadFailed);
        assert!(err.message.contains("Failed to download"));
    }

    #[test]
    #[cfg(feature = "download")]
    #[cfg_attr(miri, ignore)]
    fn test_download_model_from_url_nonexistent_dir() {
        let temp_dir = tempfile::tempdir().unwrap();
        let path = temp_dir.path().join("nonexistent/model.udpipe");
        let url = "http://localhost:1/model.udpipe";

        let result = download_model_from_url(url, &path);
        assert!(result.is_err());
    }

    #[test]
    #[cfg(feature = "download")]
    #[cfg_attr(miri, ignore)]
    fn test_download_model_from_url_empty_response() {
        let temp_dir = tempfile::tempdir().unwrap();
        let path = temp_dir.path().join("model.udpipe");

        let mut server = mockito::Server::new();
        let mock = server
            .mock("GET", "/empty-model.udpipe")
            .with_status(200)
            .with_body("")
            .create();
        let full_url = format!("{}/empty-model.udpipe", server.url());

        let result = download_model_from_url(&full_url, &path);
        mock.assert();
        drop(server);

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err.kind, UdpipeErrorKind::DownloadFailed);
        assert!(err.message.contains("empty"));
    }

    #[test]
    #[cfg_attr(miri, ignore)]
    fn test_ffi_null_sentence_word_count() {
        // SAFETY: Testing that null pointer returns 0 (defensive C++ code).
        let count = unsafe { ffi::udpipe_sentence_word_count(std::ptr::null_mut()) };
        assert_eq!(count, 0);
    }

    #[test]
    #[cfg_attr(miri, ignore)]
    fn test_ffi_null_sentence_get_word() {
        // SAFETY: Testing that null pointer returns zeroed word (defensive C++ code).
        let word = unsafe { ffi::udpipe_sentence_get_word(std::ptr::null_mut(), 0) };
        assert!(word.form.is_null());
        assert!(word.lemma.is_null());
        assert!(word.upostag.is_null());
    }

    #[test]
    #[cfg_attr(miri, ignore)]
    fn test_ffi_invalid_index() {
        // SAFETY: Testing that invalid index returns zeroed word (defensive C++ code).
        let word = unsafe { ffi::udpipe_sentence_get_word(std::ptr::null_mut(), -1) };
        assert!(word.form.is_null());
    }

    #[test]
    fn test_parser_null_returns_none() {
        let model = Model {
            inner: std::ptr::null_mut(),
        };
        let mut parser = Parser {
            inner: std::ptr::null_mut(),
            errored: false,
            _model: &model,
        };
        assert!(parser.next().is_none());
    }

    #[test]
    fn test_parser_errored_returns_none() {
        let model = Model {
            inner: std::ptr::null_mut(),
        };
        let mut parser = Parser {
            inner: std::ptr::null_mut(),
            errored: true,
            _model: &model,
        };
        assert!(parser.next().is_none());
    }

    #[test]
    #[cfg_attr(miri, ignore)]
    fn test_ffi_null_sentence_multiword_token_count() {
        // SAFETY: Testing that null pointer returns 0 (defensive C++ code).
        let count = unsafe { ffi::udpipe_sentence_multiword_token_count(std::ptr::null_mut()) };
        assert_eq!(count, 0);
    }

    #[test]
    #[cfg_attr(miri, ignore)]
    fn test_ffi_null_sentence_get_multiword_token() {
        // SAFETY: Testing that null pointer returns zeroed struct (defensive C++ code).
        let mwt = unsafe { ffi::udpipe_sentence_get_multiword_token(std::ptr::null_mut(), 0) };
        assert!(mwt.form.is_null());
        assert!(mwt.misc.is_null());
        assert_eq!(mwt.id_first, 0);
        assert_eq!(mwt.id_last, 0);
    }

    #[test]
    #[cfg_attr(miri, ignore)]
    fn test_ffi_null_sentence_comment_count() {
        // SAFETY: Testing that null pointer returns 0 (defensive C++ code).
        let count = unsafe { ffi::udpipe_sentence_comment_count(std::ptr::null_mut()) };
        assert_eq!(count, 0);
    }

    #[test]
    #[cfg_attr(miri, ignore)]
    fn test_ffi_null_sentence_get_comment() {
        // SAFETY: Testing that null pointer returns null (defensive C++ code).
        let comment = unsafe { ffi::udpipe_sentence_get_comment(std::ptr::null_mut(), 0) };
        assert!(comment.is_null());
    }

    #[test]
    fn test_ptr_to_string_null() {
        // Test that ptr_to_string returns empty string for null pointer.
        // This covers the defensive null check in ptr_to_string.
        let result = ptr_to_string(std::ptr::null());
        assert!(result.is_empty());
    }
}

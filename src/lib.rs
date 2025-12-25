//! Rust bindings for UDPipe - Universal Dependencies Pipeline.
//!
//! UDPipe is a trainable pipeline for tokenization, tagging, lemmatization,
//! and dependency parsing of CoNLL-U files.
//!
//! # Example
//!
//! ```no_run
//! use udpipe_rs::Model;
//!
//! // Download a model by language (one-time setup)
//! let model_path = udpipe_rs::download_model("english-ewt", ".")
//!     .expect("Failed to download model");
//!
//! // Load and use the model
//! let model = Model::load(&model_path).expect("Failed to load model");
//! let words = model.parse("Hello world!").expect("Failed to parse");
//!
//! for word in words {
//!     println!("{}: {} ({})", word.form, word.upostag, word.deprel);
//! }
//! ```

#![deny(missing_docs)]

use std::ffi::{CStr, CString};
use std::io::Read;
use std::path::Path;

/// Base URL for the LINDAT/CLARIAH-CZ model repository (UD 2.5).
const MODEL_BASE_URL: &str =
    "https://lindat.mff.cuni.cz/repository/xmlui/bitstream/handle/11234/1-3131";

/// Error type for UDPipe operations.
#[derive(Debug, Clone)]
pub struct UdpipeError {
    /// The error message.
    pub message: String,
}

impl UdpipeError {
    /// Create a new error with the given message.
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl std::fmt::Display for UdpipeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "UDPipe error: {}", self.message)
    }
}

impl std::error::Error for UdpipeError {}

impl From<std::io::Error> for UdpipeError {
    fn from(err: std::io::Error) -> Self {
        Self {
            message: err.to_string(),
        }
    }
}

/// A parsed word from UDPipe with Universal Dependencies annotations.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
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
    /// Miscellaneous annotations (e.g., "SpaceAfter=No").
    pub misc: String,
    /// 1-based index of this word within its sentence.
    pub id: i32,
    /// Index of the head word (0 = root).
    pub head: i32,
    /// 0-based index of the sentence this word belongs to.
    pub sentence_id: i32,
}

impl Word {
    /// Returns true if this word has a specific morphological feature.
    ///
    /// # Example
    /// ```
    /// # use udpipe_rs::Word;
    /// # let word = Word {
    /// #     form: "run".to_string(),
    /// #     lemma: "run".to_string(),
    /// #     upostag: "VERB".to_string(),
    /// #     xpostag: String::new(),
    /// #     feats: "Mood=Imp|VerbForm=Fin".to_string(),
    /// #     deprel: "root".to_string(),
    /// #     misc: String::new(),
    /// #     id: 1,
    /// #     head: 0,
    /// #     sentence_id: 0,
    /// # };
    /// assert!(word.has_feature("Mood", "Imp"));
    /// ```
    #[must_use]
    pub fn has_feature(&self, key: &str, value: &str) -> bool {
        self.get_feature(key) == Some(value)
    }

    /// Returns the value of a morphological feature, if present.
    ///
    /// # Example
    /// ```
    /// # use udpipe_rs::Word;
    /// # let word = Word {
    /// #     form: "run".to_string(),
    /// #     lemma: "run".to_string(),
    /// #     upostag: "VERB".to_string(),
    /// #     xpostag: String::new(),
    /// #     feats: "Mood=Imp|VerbForm=Fin".to_string(),
    /// #     deprel: "root".to_string(),
    /// #     misc: String::new(),
    /// #     id: 1,
    /// #     head: 0,
    /// #     sentence_id: 0,
    /// # };
    /// assert_eq!(word.get_feature("Mood"), Some("Imp"));
    /// ```
    #[must_use]
    pub fn get_feature(&self, key: &str) -> Option<&str> {
        self.feats
            .split('|')
            .find_map(|f| f.strip_prefix(key)?.strip_prefix('='))
    }

    /// Returns true if this word is a verb (VERB or AUX).
    #[must_use]
    pub fn is_verb(&self) -> bool {
        self.upostag == "VERB" || self.upostag == "AUX"
    }

    /// Returns true if this word is a noun (NOUN or PROPN).
    #[must_use]
    pub fn is_noun(&self) -> bool {
        self.upostag == "NOUN" || self.upostag == "PROPN"
    }

    /// Returns true if this word is an adjective (ADJ).
    #[must_use]
    pub fn is_adjective(&self) -> bool {
        self.upostag == "ADJ"
    }

    /// Returns true if this word is punctuation (PUNCT).
    #[must_use]
    pub fn is_punct(&self) -> bool {
        self.upostag == "PUNCT"
    }

    /// Returns true if this word is the root of its sentence.
    #[must_use]
    pub fn is_root(&self) -> bool {
        self.deprel == "root"
    }

    /// Returns true if there's a space after this word.
    ///
    /// In CoNLL-U format, `SpaceAfter=No` is only present when there's no space.
    /// This returns `true` (the default) when that annotation is absent.
    #[must_use]
    pub fn space_after(&self) -> bool {
        !self.misc.contains("SpaceAfter=No")
    }
}

// FFI declarations
mod ffi {
    use std::os::raw::c_char;

    #[repr(C)]
    pub struct UdpipeModel {
        _private: [u8; 0],
    }

    #[repr(C)]
    pub struct UdpipeParseResult {
        _private: [u8; 0],
    }

    #[repr(C)]
    pub struct UdpipeWord {
        pub form: *const c_char,
        pub lemma: *const c_char,
        pub upostag: *const c_char,
        pub xpostag: *const c_char,
        pub feats: *const c_char,
        pub deprel: *const c_char,
        pub misc: *const c_char,
        pub id: i32,
        pub head: i32,
        pub sentence_id: i32,
    }

    unsafe extern "C" {
        pub fn udpipe_model_load(model_path: *const c_char) -> *mut UdpipeModel;
        pub fn udpipe_model_load_from_memory(data: *const u8, len: usize) -> *mut UdpipeModel;
        pub fn udpipe_model_free(model: *mut UdpipeModel);
        pub fn udpipe_parse(model: *mut UdpipeModel, text: *const c_char)
        -> *mut UdpipeParseResult;
        pub fn udpipe_result_free(result: *mut UdpipeParseResult);
        pub fn udpipe_get_error() -> *const c_char;
        pub fn udpipe_result_word_count(result: *mut UdpipeParseResult) -> i32;
        pub fn udpipe_result_get_word(result: *mut UdpipeParseResult, index: i32) -> UdpipeWord;
    }
}

/// Get the last error from the FFI layer, or return a default message.
fn get_ffi_error(default: &str) -> String {
    unsafe {
        let err_ptr = ffi::udpipe_get_error();
        if err_ptr.is_null() {
            default.to_string()
        } else {
            CStr::from_ptr(err_ptr).to_string_lossy().into_owned()
        }
    }
}

/// UDPipe model wrapper.
///
/// This is the main type for loading and using UDPipe models.
/// Models can be loaded from files or from memory.
pub struct Model {
    inner: *mut ffi::UdpipeModel,
}

// SAFETY: The UDPipe model is thread-safe for parsing
unsafe impl Send for Model {}
unsafe impl Sync for Model {}

impl Model {
    /// Load a model from a file path.
    ///
    /// # Example
    /// ```no_run
    /// use udpipe_rs::Model;
    /// let model = Model::load("english-ewt-ud-2.5-191206.udpipe").expect("Failed to load model");
    /// ```
    pub fn load(path: impl AsRef<Path>) -> Result<Self, UdpipeError> {
        let path_str = path.as_ref().to_string_lossy();
        let c_path = CString::new(path_str.as_bytes()).map_err(|_| UdpipeError {
            message: "Invalid path (contains null byte)".to_string(),
        })?;

        let model = unsafe { ffi::udpipe_model_load(c_path.as_ptr()) };

        if model.is_null() {
            return Err(UdpipeError {
                message: get_ffi_error("Failed to load model"),
            });
        }

        Ok(Model { inner: model })
    }

    /// Load a model from a byte slice.
    ///
    /// This is useful for loading models from network sources or embedded data.
    ///
    /// # Example
    /// ```no_run
    /// use udpipe_rs::Model;
    /// let model_data = std::fs::read("english-ewt-ud-2.5-191206.udpipe").expect("Failed to read model");
    /// let model = Model::load_from_memory(&model_data).expect("Failed to load model");
    /// ```
    pub fn load_from_memory(data: &[u8]) -> Result<Self, UdpipeError> {
        let model = unsafe { ffi::udpipe_model_load_from_memory(data.as_ptr(), data.len()) };

        if model.is_null() {
            return Err(UdpipeError {
                message: get_ffi_error("Failed to load model from memory"),
            });
        }

        Ok(Model { inner: model })
    }

    /// Parse text and return all words with their UD annotations.
    ///
    /// The text is tokenized, tagged, lemmatized, and parsed for dependencies.
    ///
    /// # Example
    /// ```no_run
    /// use udpipe_rs::Model;
    /// let model = Model::load("english-ewt-ud-2.5-191206.udpipe").expect("Failed to load");
    /// let words = model.parse("The quick brown fox.").expect("Failed to parse");
    /// for word in words {
    ///     println!("{} -> {} ({})", word.form, word.lemma, word.upostag);
    /// }
    /// ```
    pub fn parse(&self, text: &str) -> Result<Vec<Word>, UdpipeError> {
        let c_text = CString::new(text).map_err(|_| UdpipeError {
            message: "Invalid text (contains null byte)".to_string(),
        })?;

        let result = unsafe { ffi::udpipe_parse(self.inner, c_text.as_ptr()) };

        if result.is_null() {
            return Err(UdpipeError {
                message: get_ffi_error("Failed to parse text"),
            });
        }

        let word_count = unsafe { ffi::udpipe_result_word_count(result) };
        let mut words = Vec::with_capacity(word_count as usize);

        for i in 0..word_count {
            let word = unsafe { ffi::udpipe_result_get_word(result, i) };
            words.push(Word {
                form: unsafe { CStr::from_ptr(word.form).to_string_lossy().into_owned() },
                lemma: unsafe { CStr::from_ptr(word.lemma).to_string_lossy().into_owned() },
                upostag: unsafe { CStr::from_ptr(word.upostag).to_string_lossy().into_owned() },
                xpostag: unsafe { CStr::from_ptr(word.xpostag).to_string_lossy().into_owned() },
                feats: unsafe { CStr::from_ptr(word.feats).to_string_lossy().into_owned() },
                deprel: unsafe { CStr::from_ptr(word.deprel).to_string_lossy().into_owned() },
                misc: unsafe { CStr::from_ptr(word.misc).to_string_lossy().into_owned() },
                id: word.id,
                head: word.head,
                sentence_id: word.sentence_id,
            });
        }

        unsafe { ffi::udpipe_result_free(result) };

        Ok(words)
    }
}

impl Drop for Model {
    fn drop(&mut self) {
        if !self.inner.is_null() {
            unsafe { ffi::udpipe_model_free(self.inner) };
        }
    }
}

/// Available pre-trained models from Universal Dependencies 2.5.
///
/// These models are hosted at the [LINDAT/CLARIAH-CZ repository](https://lindat.mff.cuni.cz/repository/xmlui/handle/11234/1-3131).
/// Use [`download_model`] to fetch them.
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

/// Download a pre-trained model by language identifier.
///
/// Downloads a model from the [LINDAT/CLARIAH-CZ repository](https://lindat.mff.cuni.cz/repository/xmlui/handle/11234/1-3131)
/// to the specified destination directory. Returns the path to the downloaded model file.
///
/// # Arguments
///
/// * `language` - Language identifier (e.g., "english-ewt", "dutch-alpino", "german-gsd").
///   See [`AVAILABLE_MODELS`] for the full list.
/// * `dest_dir` - Directory where the model will be saved.
///
/// # Example
///
/// ```no_run
/// use udpipe_rs::{download_model, Model};
///
/// // Download English model to current directory
/// let model_path = download_model("english-ewt", ".").expect("Failed to download");
/// println!("Model saved to: {}", model_path);
///
/// // Load and use
/// let model = Model::load(&model_path).expect("Failed to load");
/// ```
pub fn download_model(language: &str, dest_dir: impl AsRef<Path>) -> Result<String, UdpipeError> {
    let dest_dir = dest_dir.as_ref();

    // Validate the language
    if !AVAILABLE_MODELS.contains(&language) {
        return Err(UdpipeError {
            message: format!(
                "Unknown language '{}'. Use one of: {}",
                language,
                AVAILABLE_MODELS[..5].join(", ") + ", ..."
            ),
        });
    }

    // Construct filename and URL
    let filename = model_filename(language);
    let dest_path = dest_dir.join(&filename);
    let url = format!("{}/{}", MODEL_BASE_URL, filename);

    // Download using the generic download function
    download_model_from_url(&url, &dest_path)?;

    Ok(dest_path.to_string_lossy().into_owned())
}

/// Download a model from a custom URL to a local file path.
///
/// Use this if you need to download models from a different source or version.
/// For standard models, prefer [`download_model`].
///
/// # Example
///
/// ```no_run
/// use udpipe_rs::download_model_from_url;
///
/// download_model_from_url(
///     "https://example.com/custom-model.udpipe",
///     "custom-model.udpipe",
/// ).expect("Failed to download");
/// ```
pub fn download_model_from_url(url: &str, path: impl AsRef<Path>) -> Result<(), UdpipeError> {
    let path = path.as_ref();

    // Create parent directories if needed
    if let Some(parent) = path.parent() {
        if !parent.as_os_str().is_empty() {
            std::fs::create_dir_all(parent)?;
        }
    }

    // Download using ureq
    let response = ureq::get(url).call().map_err(|e| UdpipeError {
        message: format!("Failed to download: {}", e),
    })?;

    // Read response body
    let mut data = Vec::new();
    response
        .into_body()
        .into_reader()
        .read_to_end(&mut data)
        .map_err(|e| UdpipeError {
            message: format!("Failed to read response: {}", e),
        })?;

    if data.is_empty() {
        return Err(UdpipeError {
            message: "Downloaded file is empty".to_string(),
        });
    }

    // Write to file
    std::fs::write(path, &data)?;

    Ok(())
}

/// Returns the expected filename for a given language model.
///
/// # Example
///
/// ```
/// assert_eq!(udpipe_rs::model_filename("english-ewt"), "english-ewt-ud-2.5-191206.udpipe");
/// ```
pub fn model_filename(language: &str) -> String {
    format!("{}-ud-2.5-191206.udpipe", language)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_word(feats: &str) -> Word {
        Word {
            form: "test".to_string(),
            lemma: "test".to_string(),
            upostag: "NOUN".to_string(),
            xpostag: String::new(),
            feats: feats.to_string(),
            deprel: "root".to_string(),
            misc: String::new(),
            id: 1,
            head: 0,
            sentence_id: 0,
        }
    }

    #[test]
    fn test_word_has_feature() {
        let word = make_word("Mood=Imp|VerbForm=Fin");

        assert!(word.has_feature("Mood", "Imp"));
        assert!(word.has_feature("VerbForm", "Fin"));
        assert!(!word.has_feature("Mood", "Ind"));
        assert!(!word.has_feature("Tense", "Past"));
    }

    #[test]
    fn test_word_has_feature_empty() {
        let word = make_word("");
        assert!(!word.has_feature("Mood", "Imp"));
    }

    #[test]
    fn test_word_has_feature_single() {
        let word = make_word("Mood=Imp");
        assert!(word.has_feature("Mood", "Imp"));
        assert!(!word.has_feature("VerbForm", "Fin"));
    }

    #[test]
    fn test_word_get_feature() {
        let word = make_word("Tense=Pres|VerbForm=Part");

        assert_eq!(word.get_feature("Tense"), Some("Pres"));
        assert_eq!(word.get_feature("VerbForm"), Some("Part"));
        assert_eq!(word.get_feature("Mood"), None);
    }

    #[test]
    fn test_word_get_feature_empty() {
        let word = make_word("");
        assert_eq!(word.get_feature("Mood"), None);
    }

    #[test]
    fn test_word_get_feature_single() {
        let word = make_word("Mood=Imp");
        assert_eq!(word.get_feature("Mood"), Some("Imp"));
        assert_eq!(word.get_feature("VerbForm"), None);
    }

    #[test]
    fn test_word_is_verb() {
        let mut word = make_word("");
        word.upostag = "VERB".to_string();
        assert!(word.is_verb());

        word.upostag = "AUX".to_string();
        assert!(word.is_verb());

        word.upostag = "NOUN".to_string();
        assert!(!word.is_verb());
    }

    #[test]
    fn test_word_is_noun() {
        let mut word = make_word("");
        word.upostag = "NOUN".to_string();
        assert!(word.is_noun());

        word.upostag = "PROPN".to_string();
        assert!(word.is_noun());

        word.upostag = "VERB".to_string();
        assert!(!word.is_noun());
    }

    #[test]
    fn test_word_is_root() {
        let mut word = make_word("");
        word.deprel = "root".to_string();
        assert!(word.is_root());

        word.deprel = "nsubj".to_string();
        assert!(!word.is_root());
    }

    #[test]
    fn test_word_is_adjective() {
        let mut word = make_word("");
        word.upostag = "ADJ".to_string();
        assert!(word.is_adjective());

        word.upostag = "NOUN".to_string();
        assert!(!word.is_adjective());
    }

    #[test]
    fn test_word_is_punct() {
        let mut word = make_word("");
        word.upostag = "PUNCT".to_string();
        assert!(word.is_punct());

        word.upostag = "NOUN".to_string();
        assert!(!word.is_punct());
    }

    #[test]
    fn test_word_hash() {
        use std::collections::HashSet;

        let word1 = make_word("Mood=Imp");
        let word2 = make_word("Mood=Imp");
        let mut set = HashSet::new();
        set.insert(word1.clone());
        assert!(set.contains(&word2));
    }

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
        // Verify the list is sorted for binary search if needed later
        let mut sorted = AVAILABLE_MODELS.to_vec();
        sorted.sort();
        assert_eq!(AVAILABLE_MODELS, sorted.as_slice());
    }

    #[test]
    fn test_download_model_invalid_language() {
        let result = download_model("invalid-language-xyz", ".");
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.message.contains("Unknown language"));
    }

    #[test]
    fn test_udpipe_error_display() {
        let err = UdpipeError::new("test error");
        assert_eq!(format!("{}", err), "UDPipe error: test error");
    }

    #[test]
    fn test_udpipe_error_from_io() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let err: UdpipeError = io_err.into();
        assert!(err.message.contains("not found"));
    }

    #[test]
    fn test_space_after() {
        let mut word = make_word("");
        word.misc = String::new();
        assert!(word.space_after()); // default: has space

        word.misc = "SpaceAfter=No".to_string();
        assert!(!word.space_after());

        word.misc = "SpaceAfter=No|Other=Value".to_string();
        assert!(!word.space_after());
    }
}

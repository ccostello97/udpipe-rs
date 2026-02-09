#ifndef UDPIPE_WRAPPER_H
#define UDPIPE_WRAPPER_H

#include <cstddef>
#include <cstdint>

#ifdef __cplusplus
extern "C" {
#endif

// Opaque types (defined in .cpp)
struct UdpipeModel;
struct UdpipeParser;
struct UdpipeSentence;

// Word structure with Universal Dependencies annotations.
// Note: The virtual root word (index 0 in UDPipe) is excluded from results.
// All string pointers are valid only until the next sentence API call or
// udpipe_sentence_free on the same UdpipeSentence.
struct UdpipeWord {
  const char *form;        // Surface form
  const char *lemma;       // Lemma (dictionary form)
  const char *upostag;     // Universal POS tag
  const char *xpostag;     // Language-specific POS tag
  const char *feats;       // Morphological features
  const char *deprel;      // Dependency relation
  const char *deps;        // Enhanced dependencies
  const char *misc;        // Miscellaneous (e.g., SpaceAfter=No)
  const int32_t *children; // Pointer to array of child word IDs
  int32_t id;              // 1-based word index within sentence
  int32_t head;            // Head word index (0 = root)
  int32_t children_count;  // Number of children
};

// Multiword token (e.g., "don't" -> "do" + "n't").
// String pointers valid until next sentence API call or udpipe_sentence_free.
struct UdpipeMultiwordToken {
  const char *form; // Surface form of the multiword token
  const char *misc; // Miscellaneous annotations
  int32_t id_first; // First word ID in the token range
  int32_t id_last;  // Last word ID in the token range
};

// Model functions
// On failure, return nullptr. If out_error != nullptr, set *out_error to the
// last error message (valid only until the next API call on this thread; copy
// immediately).
auto udpipe_model_load(const char *model_path, const char **out_error)
    -> UdpipeModel *;
auto udpipe_model_load_from_memory(const uint8_t *data, size_t len,
                                   const char **out_error) -> UdpipeModel *;
void udpipe_model_free(UdpipeModel *model);

// Parser functions - streaming API
// On failure, return nullptr. If out_error != nullptr, set *out_error to the
// error message (valid until next API call on this thread).
auto udpipe_parser_new(UdpipeModel *model, const char *text, size_t text_len,
                       const char **out_error) -> UdpipeParser *;
auto udpipe_parser_next(UdpipeParser *parser, const char **out_error)
    -> UdpipeSentence *;
auto udpipe_parser_has_error(UdpipeParser *parser) -> bool;
void udpipe_parser_free(UdpipeParser *parser);

// Sentence functions - words
void udpipe_sentence_free(UdpipeSentence *sentence);
auto udpipe_sentence_word_count(UdpipeSentence *sentence) -> int32_t;
auto udpipe_sentence_get_word(UdpipeSentence *sentence, int32_t index)
    -> UdpipeWord;

// Sentence functions - multiword tokens
auto udpipe_sentence_multiword_token_count(UdpipeSentence *sentence) -> int32_t;
auto udpipe_sentence_get_multiword_token(UdpipeSentence *sentence,
                                         int32_t index) -> UdpipeMultiwordToken;

// Sentence functions - comments
auto udpipe_sentence_comment_count(UdpipeSentence *sentence) -> int32_t;
// Returned pointer valid until next sentence API call or udpipe_sentence_free.
auto udpipe_sentence_get_comment(UdpipeSentence *sentence, int32_t index)
    -> const char *;

#ifdef __cplusplus
}
#endif

#endif // UDPIPE_WRAPPER_H

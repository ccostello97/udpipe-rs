#ifndef UDPIPE_WRAPPER_H
#define UDPIPE_WRAPPER_H

#include <cstddef>
#include <cstdint>

#ifdef __cplusplus
extern "C" {
#endif

// Opaque types
using UdpipeModel = struct UdpipeModel;
using UdpipeParseResult = struct UdpipeParseResult;

// Word structure with Universal Dependencies annotations
using UdpipeWord = struct {
  const char *form;    // Surface form
  const char *lemma;   // Lemma (dictionary form)
  const char *upostag; // Universal POS tag
  const char *xpostag; // Language-specific POS tag
  const char *feats;   // Morphological features
  const char *deprel;  // Dependency relation
  const char *misc;    // Miscellaneous (e.g., SpaceAfter=No)
  int32_t id;          // 1-based word index within sentence
  int32_t head;        // Head word index (0 = root)
  int32_t sentence_id; // 0-based sentence index
};

// Model functions
auto udpipe_model_load(const char *model_path) -> UdpipeModel *;
auto udpipe_model_load_from_memory(const uint8_t *data,
                                   size_t len) -> UdpipeModel *;
void udpipe_model_free(UdpipeModel *model);

// Parse function - returns a result that must be freed with udpipe_result_free
auto udpipe_parse(UdpipeModel *model, const char *text) -> UdpipeParseResult *;

// Result functions
void udpipe_result_free(UdpipeParseResult *result);
auto udpipe_result_word_count(UdpipeParseResult *result) -> int32_t;
auto udpipe_result_get_word(UdpipeParseResult *result,
                            int32_t index) -> UdpipeWord;

// Error handling
auto udpipe_get_error() -> const char *;

#ifdef __cplusplus
}
#endif

#endif // UDPIPE_WRAPPER_H

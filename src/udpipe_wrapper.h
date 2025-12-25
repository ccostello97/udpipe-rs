#ifndef UDPIPE_WRAPPER_H
#define UDPIPE_WRAPPER_H

#include <stddef.h>
#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

// Opaque types
typedef struct UdpipeModel UdpipeModel;
typedef struct UdpipeParseResult UdpipeParseResult;

// Word structure with Universal Dependencies annotations
typedef struct {
    const char* form;    // Surface form
    const char* lemma;   // Lemma (dictionary form)
    const char* upostag; // Universal POS tag
    const char* xpostag; // Language-specific POS tag
    const char* feats;   // Morphological features
    const char* deprel;  // Dependency relation
    const char* misc;    // Miscellaneous (e.g., SpaceAfter=No)
    int32_t id;          // 1-based word index within sentence
    int32_t head;        // Head word index (0 = root)
    int32_t sentence_id; // 0-based sentence index
} UdpipeWord;

// Model functions
UdpipeModel* udpipe_model_load(const char* model_path);
UdpipeModel* udpipe_model_load_from_memory(const uint8_t* data, size_t len);
void udpipe_model_free(UdpipeModel* model);

// Parse function - returns a result that must be freed with udpipe_result_free
UdpipeParseResult* udpipe_parse(UdpipeModel* model, const char* text);

// Result functions
void udpipe_result_free(UdpipeParseResult* result);
int32_t udpipe_result_word_count(UdpipeParseResult* result);
UdpipeWord udpipe_result_get_word(UdpipeParseResult* result, int32_t index);

// Error handling
const char* udpipe_get_error(void);

#ifdef __cplusplus
}
#endif

#endif // UDPIPE_WRAPPER_H

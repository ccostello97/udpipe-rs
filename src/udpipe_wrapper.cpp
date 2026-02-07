#include "udpipe_wrapper.h"
#include "model/model.h"
#include "sentence/input_format.h"
#include "sentence/sentence.h"

#include <cstddef>
#include <cstdint>
#include <cstdlib>
#include <cstring>
#include <istream>
#include <memory>
#include <streambuf>
#include <string>
#include <utility>
#include <vector>

using ufal::udpipe::input_format;
using ufal::udpipe::model;
using ufal::udpipe::sentence;

namespace {
// Thread-local error message. *out_error points into this; valid until next API
// call.
auto last_error() -> std::string & {
  static thread_local std::string error;
  return error;
}

// std::streambuf that reads from (data, len) without copying. setg() requires
// non-const pointers; we only read from the get area, so the cast is safe.
class memory_streambuf : public std::streambuf {
public:
  memory_streambuf(const char *data, std::size_t len) {
    char *base = const_cast<char *>(data);
    setg(base, base, base + len);
  }
};
} // namespace

struct UdpipeModel {
  std::unique_ptr<model> m;
};

// Streaming parser that yields one sentence at a time
struct UdpipeParser {
  UdpipeModel *model = nullptr;
  std::unique_ptr<input_format> tokenizer;
  bool finished = false;
  bool errored = false;
};

// Single sentence with all data from UDPipe
struct UdpipeSentence {
  std::vector<std::string> forms;
  std::vector<std::string> lemmas;
  std::vector<std::string> upostags;
  std::vector<std::string> xpostags;
  std::vector<std::string> feats;
  std::vector<std::string> deprels;
  std::vector<std::string> deps;
  std::vector<std::string> miscs;
  std::vector<int32_t> ids;
  std::vector<int32_t> heads;
  std::vector<int32_t> children_flat;
  std::vector<int32_t> children_offsets;
  std::vector<int32_t> children_counts;
  std::vector<std::string> mwt_forms;
  std::vector<std::string> mwt_miscs;
  std::vector<int32_t> mwt_id_firsts;
  std::vector<int32_t> mwt_id_lasts;
  std::vector<std::string> comments;
};

namespace {
auto build_sentence(const sentence &current_sentence) -> UdpipeSentence * {
  auto *result = new UdpipeSentence();
  size_t const word_count =
      !current_sentence.words.empty() ? current_sentence.words.size() - 1 : 0;
  result->forms.reserve(word_count);
  result->lemmas.reserve(word_count);
  result->upostags.reserve(word_count);
  result->xpostags.reserve(word_count);
  result->feats.reserve(word_count);
  result->deprels.reserve(word_count);
  result->deps.reserve(word_count);
  result->miscs.reserve(word_count);
  result->ids.reserve(word_count);
  result->heads.reserve(word_count);
  result->children_offsets.reserve(word_count);
  result->children_counts.reserve(word_count);

  for (size_t idx = 1; idx < current_sentence.words.size(); idx++) {
    const auto &word = current_sentence.words[idx];
    result->forms.push_back(word.form);
    result->lemmas.push_back(word.lemma);
    result->upostags.push_back(word.upostag);
    result->xpostags.push_back(word.xpostag);
    result->feats.push_back(word.feats);
    result->deprels.push_back(word.deprel);
    result->deps.push_back(word.deps);
    result->miscs.push_back(word.misc);
    result->ids.push_back(static_cast<int32_t>(word.id));
    result->heads.push_back(word.head);
    result->children_offsets.push_back(
        static_cast<int32_t>(result->children_flat.size()));
    result->children_counts.push_back(
        static_cast<int32_t>(word.children.size()));
    for (int child_id : word.children) {
      result->children_flat.push_back(static_cast<int32_t>(child_id));
    }
  }

  for (const auto &mwt : current_sentence.multiword_tokens) {
    result->mwt_forms.push_back(mwt.form);
    result->mwt_miscs.push_back(mwt.misc);
    result->mwt_id_firsts.push_back(static_cast<int32_t>(mwt.id_first));
    result->mwt_id_lasts.push_back(static_cast<int32_t>(mwt.id_last));
  }
  result->comments = current_sentence.comments;
  return result;
}
} // namespace

auto udpipe_model_load(const char *model_path, const char **out_error)
    -> UdpipeModel * {
  last_error().clear();

  std::unique_ptr<model> loaded_model(model::load(model_path));
  if (!loaded_model) {
    last_error() = "Failed to load model from: ";
    last_error() += model_path;
    if (out_error != nullptr) {
      *out_error = last_error().c_str();
    }
    return nullptr;
  }

  auto *wrapper = new UdpipeModel();
  wrapper->m = std::move(loaded_model);
  return wrapper;
}

auto udpipe_model_load_from_memory(const uint8_t *data, size_t len,
                                   const char **out_error) -> UdpipeModel * {
  last_error().clear();

  memory_streambuf buf(reinterpret_cast<const char *>(data), len);
  std::istream model_stream(&buf);

  std::unique_ptr<model> loaded_model(model::load(model_stream));
  if (!loaded_model) {
    last_error() = "Failed to load model from memory";
    if (out_error != nullptr) {
      *out_error = last_error().c_str();
    }
    return nullptr;
  }

  auto *wrapper = new UdpipeModel();
  wrapper->m = std::move(loaded_model);
  return wrapper;
}

void udpipe_model_free(UdpipeModel *model) { delete model; }

auto udpipe_parser_new(UdpipeModel *model, const char *text,
                       const char **out_error) -> UdpipeParser * {
  if (model == nullptr || !model->m || text == nullptr) {
    last_error() = "Invalid arguments to udpipe_parser_new";
    if (out_error != nullptr) {
      *out_error = last_error().c_str();
    }
    return nullptr;
  }

  last_error().clear();

  std::unique_ptr<input_format> tokenizer(
      model->m->new_tokenizer(model::DEFAULT));
  if (!tokenizer) {
    last_error() = "Failed to create tokenizer";
    if (out_error != nullptr) {
      *out_error = last_error().c_str();
    }
    return nullptr;
  }

  // Set text to tokenize (make_copy=true ensures text is copied since
  // the original string may be deallocated before parsing completes)
  tokenizer->set_text(text, true);

  auto *parser = new UdpipeParser();
  parser->model = model;
  parser->tokenizer = std::move(tokenizer);
  parser->finished = false;

  return parser;
}

auto udpipe_parser_next(UdpipeParser *parser, const char **out_error)
    -> UdpipeSentence * {
  if (parser == nullptr || parser->finished) {
    if (out_error != nullptr) {
      *out_error = nullptr;
    }
    return nullptr;
  }

  sentence current_sentence;
  std::string error;

  if (!parser->tokenizer->next_sentence(current_sentence, error)) {
    parser->finished = true;
    if (!error.empty()) {
      last_error() = error;
      parser->errored = true;
      if (out_error != nullptr) {
        *out_error = last_error().c_str();
      }
    } else {
      if (out_error != nullptr) {
        *out_error = nullptr;
      }
    }
    return nullptr;
  }

  parser->model->m->tag(current_sentence, model::DEFAULT, error);
  if (!error.empty()) {
    last_error() = error;
    parser->finished = true;
    parser->errored = true;
    if (out_error != nullptr) {
      *out_error = last_error().c_str();
    }
    return nullptr;
  }

  parser->model->m->parse(current_sentence, model::DEFAULT, error);
  if (!error.empty()) {
    last_error() = error;
    parser->finished = true;
    parser->errored = true;
    if (out_error != nullptr) {
      *out_error = last_error().c_str();
    }
    return nullptr;
  }

  return build_sentence(current_sentence);
}

auto udpipe_parser_has_error(UdpipeParser *parser) -> bool {
  return parser != nullptr && parser->errored;
}

void udpipe_parser_free(UdpipeParser *parser) { delete parser; }

void udpipe_sentence_free(UdpipeSentence *sentence) { delete sentence; }

auto udpipe_sentence_word_count(UdpipeSentence *sentence) -> int32_t {
  if (sentence == nullptr) {
    return 0;
  }
  return static_cast<int32_t>(sentence->forms.size());
}

auto udpipe_sentence_get_word(UdpipeSentence *sentence, int32_t index)
    -> UdpipeWord {
  UdpipeWord word = {}; // Zero-initialize all fields

  if (sentence == nullptr || index < 0 ||
      static_cast<size_t>(index) >= sentence->forms.size()) {
    return word;
  }

  auto idx = static_cast<size_t>(index);
  word.form = sentence->forms[idx].c_str();
  word.lemma = sentence->lemmas[idx].c_str();
  word.upostag = sentence->upostags[idx].c_str();
  word.xpostag = sentence->xpostags[idx].c_str();
  word.feats = sentence->feats[idx].c_str();
  word.deprel = sentence->deprels[idx].c_str();
  word.deps = sentence->deps[idx].c_str();
  word.misc = sentence->miscs[idx].c_str();
  word.id = sentence->ids[idx];
  word.head = sentence->heads[idx];

  // Children pointer and count
  int32_t const offset = sentence->children_offsets[idx];
  word.children_count = sentence->children_counts[idx];
  word.children =
      word.children_count > 0 ? &sentence->children_flat[offset] : nullptr;

  return word;
}

auto udpipe_sentence_multiword_token_count(UdpipeSentence *sentence)
    -> int32_t {
  if (sentence == nullptr) {
    return 0;
  }
  return static_cast<int32_t>(sentence->mwt_forms.size());
}

auto udpipe_sentence_get_multiword_token(UdpipeSentence *sentence,
                                         int32_t index)
    -> UdpipeMultiwordToken {
  UdpipeMultiwordToken mwt = {}; // Zero-initialize all fields

  if (sentence == nullptr || index < 0 ||
      static_cast<size_t>(index) >= sentence->mwt_forms.size()) {
    return mwt;
  }

  // Covered by Spanish model integration test
  auto idx = static_cast<size_t>(index);
  mwt.form = sentence->mwt_forms[idx].c_str();
  mwt.misc = sentence->mwt_miscs[idx].c_str();
  mwt.id_first = sentence->mwt_id_firsts[idx];
  mwt.id_last = sentence->mwt_id_lasts[idx];

  return mwt;
}

auto udpipe_sentence_comment_count(UdpipeSentence *sentence) -> int32_t {
  if (sentence == nullptr) {
    return 0;
  }
  return static_cast<int32_t>(sentence->comments.size());
}

auto udpipe_sentence_get_comment(UdpipeSentence *sentence, int32_t index)
    -> const char * {
  if (sentence == nullptr || index < 0 ||
      static_cast<size_t>(index) >= sentence->comments.size()) {
    return nullptr;
  }
  return sentence->comments[static_cast<size_t>(index)].c_str();
}

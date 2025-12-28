#include "udpipe_wrapper.h"
#include "model/model.h"
#include "sentence/input_format.h"
#include "sentence/sentence.h"

#include <memory>
#include <sstream>
#include <string>
#include <vector>

using ufal::udpipe::input_format;
using ufal::udpipe::model;
using ufal::udpipe::sentence;

// Thread-local error message accessor (avoids global mutable state)
auto last_error() -> std::string & {
  static thread_local std::string error;
  return error;
}

struct UdpipeModel {
  std::unique_ptr<model> m;
};

// Flattened word data for O(1) access
struct UdpipeParseResult {
  std::vector<std::string> forms;
  std::vector<std::string> lemmas;
  std::vector<std::string> upostags;
  std::vector<std::string> xpostags;
  std::vector<std::string> feats;
  std::vector<std::string> deprels;
  std::vector<std::string> miscs;
  std::vector<int32_t> ids;
  std::vector<int32_t> heads;
  std::vector<int32_t> sentence_ids;
};

auto udpipe_model_load(const char *model_path) -> UdpipeModel * {
  last_error().clear();

  std::unique_ptr<model> loaded_model(model::load(model_path));
  if (!loaded_model) {
    last_error() = "Failed to load model from: ";
    last_error() += model_path;
    return nullptr;
  }

  auto *wrapper = new UdpipeModel();
  wrapper->m = std::move(loaded_model);
  return wrapper;
}

auto udpipe_model_load_from_memory(const uint8_t *data, size_t len) -> UdpipeModel * {
  last_error().clear();

  std::string model_data(reinterpret_cast<const char *>(data), len);
  std::istringstream model_stream(model_data);

  std::unique_ptr<model> loaded_model(model::load(model_stream));
  if (!loaded_model) {
    last_error() = "Failed to load model from memory";
    return nullptr;
  }

  auto *wrapper = new UdpipeModel();
  wrapper->m = std::move(loaded_model);
  return wrapper;
}

void udpipe_model_free(UdpipeModel *model) { delete model; }

auto udpipe_parse(UdpipeModel *model, const char *text) -> UdpipeParseResult * {
  if (model == nullptr || !model->m || text == nullptr) {
    last_error() = "Invalid arguments to udpipe_parse";
    return nullptr;
  }

  last_error().clear();

  auto *result = new UdpipeParseResult();

  // Create input format (tokenizer)
  std::unique_ptr<input_format> tokenizer(model->m->new_tokenizer(model::DEFAULT));
  if (!tokenizer) {
    last_error() = "Failed to create tokenizer";
    delete result;
    return nullptr;
  }

  // Set text to tokenize
  tokenizer->set_text(text);

  // Parse sentences and extract word data directly
  sentence current_sentence;
  std::string error;
  int32_t sentence_idx = 0;

  while (tokenizer->next_sentence(current_sentence, error)) {
    // Tag and parse
    model->m->tag(current_sentence, model::DEFAULT, error);
    model->m->parse(current_sentence, model::DEFAULT, error);

    // Extract word data (skip root at index 0)
    for (size_t idx = 1; idx < current_sentence.words.size(); idx++) {
      const auto &word = current_sentence.words[idx];
      result->forms.push_back(word.form);
      result->lemmas.push_back(word.lemma);
      result->upostags.push_back(word.upostag);
      result->xpostags.push_back(word.xpostag);
      result->feats.push_back(word.feats);
      result->deprels.push_back(word.deprel);
      result->miscs.push_back(word.misc);
      result->ids.push_back(static_cast<int32_t>(word.id));
      result->heads.push_back(word.head);
      result->sentence_ids.push_back(sentence_idx);
    }
    sentence_idx++;
  }

  if (!error.empty()) {
    last_error() = error;
    delete result;
    return nullptr;
  }

  return result;
}

void udpipe_result_free(UdpipeParseResult *result) { delete result; }

auto udpipe_result_word_count(UdpipeParseResult *result) -> int32_t {
  if (result == nullptr) {
    return 0;
  }
  return static_cast<int32_t>(result->forms.size());
}

auto udpipe_result_get_word(UdpipeParseResult *result, int32_t index) -> UdpipeWord {
  UdpipeWord word = {nullptr, nullptr, nullptr, nullptr, nullptr, nullptr, nullptr, 0, 0, 0};

  if (result == nullptr || index < 0 || static_cast<size_t>(index) >= result->forms.size()) {
    return word;
  }

  auto idx = static_cast<size_t>(index);
  word.form = result->forms[idx].c_str();
  word.lemma = result->lemmas[idx].c_str();
  word.upostag = result->upostags[idx].c_str();
  word.xpostag = result->xpostags[idx].c_str();
  word.feats = result->feats[idx].c_str();
  word.deprel = result->deprels[idx].c_str();
  word.misc = result->miscs[idx].c_str();
  word.id = result->ids[idx];
  word.head = result->heads[idx];
  word.sentence_id = result->sentence_ids[idx];

  return word;
}

auto udpipe_get_error() -> const char * {
  return last_error().empty() ? nullptr : last_error().c_str();
}

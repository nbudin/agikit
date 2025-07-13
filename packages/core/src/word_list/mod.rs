use std::collections::{HashMap, HashSet};

use wasm_bindgen::prelude::wasm_bindgen;

pub mod decode;
pub mod encode;
pub mod words_txt;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
#[wasm_bindgen]
pub struct WordList {
    #[wasm_bindgen(skip)]
    pub words: HashMap<u16, WordListEntry>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WordListEntry {
    pub number: u16,
    pub words: HashSet<String>,
    pub canonical_word: String,
}

impl WordListEntry {
    fn iter_words(&self) -> impl Iterator<Item = &str> {
        std::iter::once(self.canonical_word.as_str()).chain(
            self.words
                .iter()
                .filter(|word| **word != self.canonical_word)
                .map(|word| word.as_str()),
        )
    }
}

#[cfg(test)]
mod tests {
    use similar_asserts::assert_eq;
    use std::collections::HashSet;

    use crate::{
        resources::{decode::Decode, encode::Encode, file_provider::FileProvider},
        test_data::contest2_template_dir,
        word_list::{WordList, WordListEntry},
    };

    #[test]
    fn smoke_test() {
        let words_tok_data = contest2_template_dir()
            .read_file_bytes("WORDS.TOK")
            .expect("Failed to get WORDS.TOK file");
        let word_list =
            WordList::decode_from_bytes(&words_tok_data, ()).expect("Failed to decode WORDS.TOK");

        assert_eq!(
            word_list.words.get(&9999),
            Some(&WordListEntry {
                number: 9999,
                canonical_word: "rol".to_string(),
                words: HashSet::from(["rol".to_string()])
            })
        );

        let encoded = word_list
            .encode_to_vec(())
            .expect("Failed to encode WordList");
        assert_eq!(words_tok_data, encoded);
    }
}

#[cfg(feature = "js")]
pub mod js {
    use std::collections::HashMap;

    use wasm_bindgen::{JsValue, prelude::wasm_bindgen};
    use web_sys::js_sys;

    use crate::word_list::{WordList, WordListEntry};

    impl Into<JsValue> for WordListEntry {
        fn into(self) -> JsValue {
            let pair = js_sys::Array::new();
            pair.push(&JsValue::from(self.number));
            let words = self
                .iter_words()
                .map(JsValue::from)
                .collect::<js_sys::Array>();
            pair.push(&words);
            pair.into()
        }
    }

    #[wasm_bindgen]
    impl WordList {
        #[wasm_bindgen(constructor)]
        pub fn new() -> Self {
            WordList {
                words: HashMap::new(),
            }
        }

        #[wasm_bindgen(js_name = "entries")]
        pub fn js_entries(&self) -> Vec<JsValue> {
            self.words.values().cloned().map(Into::into).collect()
        }

        #[wasm_bindgen(js_name = "get")]
        pub fn js_get(&self, word_number: u16) -> Option<js_sys::Array> {
            self.words
                .get(&word_number)
                .map(|entry| js_sys::Array::from_iter(entry.iter_words().map(JsValue::from)))
        }
    }
}

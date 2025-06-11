use std::collections::{HashMap, HashSet};

use wasm_bindgen::{prelude::wasm_bindgen, JsValue};
use web_sys::js_sys;

pub mod decode;
pub mod encode;
pub mod words_txt;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
#[wasm_bindgen]
pub struct WordList {
    #[wasm_bindgen(skip)]
    pub words: HashMap<u16, HashSet<String>>,
}

struct WordListEntry(u16, Vec<String>);

impl Into<JsValue> for WordListEntry {
    fn into(self) -> JsValue {
        let pair = js_sys::Array::new();
        pair.push(&JsValue::from(self.0));
        let words = js_sys::Array::from(&self.1.into());
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

    pub fn entries(&self) -> Vec<JsValue> {
        self.words
            .iter()
            .map(|(key, value)| WordListEntry(*key, value.iter().cloned().collect()).into())
            .collect()
    }

    pub fn get(&self, word_number: u16) -> Option<js_sys::Array> {
        self.words
            .get(&word_number)
            .map(|set| js_sys::Array::from_iter(set.iter().map(JsValue::from)))
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;
    use std::collections::HashSet;

    use crate::{
        resources::{decode::Decode, encode::Encode},
        word_list::WordList,
        TEST_DATA_DIR,
    };

    #[test]
    fn smoke_test() {
        let words_tok_data = TEST_DATA_DIR
            .get_file("AGI_Contest_2_Template/WORDS.TOK")
            .expect("Failed to get WORDS.TOK file")
            .contents();
        let word_list = WordList::decode(&mut std::io::Cursor::new(words_tok_data), ())
            .expect("Failed to decode WORDS.TOK");

        assert_eq!(
            word_list.words.get(&9999),
            Some(&HashSet::from(["rol".to_string()]))
        );

        let encoded = word_list.encode(()).expect("Failed to encode WordList");
        assert_eq!(words_tok_data, encoded);
    }
}

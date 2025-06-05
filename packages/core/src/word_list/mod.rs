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
        resource::{Decode, Encode},
        word_list::WordList,
    };

    static WORDS_TOK: &[u8] = include_bytes!("../../test_data/WORDS.TOK");

    #[test]
    fn smoke_test() {
        let word_list = WordList::decode(&mut std::io::Cursor::new(WORDS_TOK), ())
            .expect("Failed to decode WORDS.TOK");

        assert_eq!(
            word_list.words.get(&9999),
            Some(&HashSet::from(["rol".to_string()]))
        );

        let encoded = word_list.encode(()).expect("Failed to encode WordList");
        assert_eq!(WORDS_TOK, encoded);
    }
}

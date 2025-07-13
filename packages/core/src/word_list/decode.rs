use std::{
    collections::HashSet,
    io::{Cursor, SeekFrom},
};

use wasm_bindgen::{JsValue, prelude::wasm_bindgen};
use web_sys::js_sys::Uint8Array;

use crate::{
    buffer::Buffer, data_encoding::ReadHeterogeneousData, resources::decode::Decode,
    word_list::WordListEntry,
};

use super::WordList;

impl Decode<'_> for WordList {
    type Options = ();

    fn decode<'a, Data: ReadHeterogeneousData>(
        data: &'a mut Data,
        _options: Self::Options,
    ) -> Result<Self, crate::resources::decode::DecodingError>
    where
        Self: Sized,
    {
        let mut word_list = WordList::new();
        data.seek(SeekFrom::Start(52))?; // skip header

        let mut previous_word: Vec<char> = vec![];
        let mut current_word: Vec<char>;

        loop {
            let Ok(reuse_chars) = data.read_u8() else {
                break;
            };

            current_word = previous_word.split_at(reuse_chars as usize).0.to_vec();

            loop {
                let Ok(char) = data.read_u8() else {
                    break;
                };

                if char > 0x7f {
                    // high bytes signify end of word
                    current_word.push(((char ^ 0x7f) & 0x7f) as char);
                    previous_word = current_word.clone();
                    let word_number = data.read_u16_be()?;

                    let current_word_str = current_word.iter().collect::<String>();

                    let entry =
                        word_list
                            .words
                            .entry(word_number)
                            .or_insert_with(|| WordListEntry {
                                words: HashSet::new(),
                                canonical_word: current_word_str.clone(),
                                number: word_number,
                            });
                    entry.words.insert(current_word.iter().collect::<String>());
                    break;
                } else {
                    current_word.push((char ^ 0x7f) as char);
                }
            }
        }

        Ok(word_list)
    }
}

#[wasm_bindgen(js_name = readWordsTok)]
pub fn read_words_tok(data: Buffer) -> Result<WordList, JsValue> {
    let data_array = Uint8Array::new(&data);
    let data_vec = data_array.to_vec();
    WordList::decode(&mut Cursor::new(data_vec), ())
        .map_err(|e| JsValue::from_str(format!("{:?}", e).as_str()))
}

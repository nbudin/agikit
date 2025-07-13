use std::collections::HashMap;

use crate::{
    data_encoding::{WriteHeterogeneousData, encode_uint16be},
    resources::encode::{Encode, EncodingError},
    word_list::WordList,
};

static LETTERS: [char; 26] = [
    'A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'I', 'J', 'K', 'L', 'M', 'N', 'O', 'P', 'Q', 'R', 'S',
    'T', 'U', 'V', 'W', 'X', 'Y', 'Z',
];

fn encode_word(word: &str) -> impl Iterator<Item = u8> + '_ {
    let length = word.len();
    word.chars().enumerate().map(move |(index, c)| {
        let masked_char = (c as u8) ^ 0x7f;
        if index == length - 1 {
            masked_char | 0x80
        } else {
            masked_char
        }
    })
}

impl Encode<'_> for WordList {
    type Options = ();

    fn encode<Out: WriteHeterogeneousData>(
        &self,
        mut out: Out,
        _options: Self::Options,
    ) -> Result<(), EncodingError> {
        let number_by_word = self
            .words
            .iter()
            .flat_map(|(word_number, entry)| {
                entry
                    .iter_words()
                    .filter(|word| *word != "ANYWORD")
                    .map(|word| (word, *word_number))
            })
            .collect::<HashMap<_, _>>();

        let mut sorted_words = number_by_word.keys().cloned().collect::<Vec<_>>();
        sorted_words.sort_unstable();

        let mut words_entries_by_first_letter: HashMap<char, Vec<Vec<u8>>> = LETTERS
            .iter()
            .map(|&letter| (letter, vec![]))
            .collect::<HashMap<_, _>>();

        let mut previous_word: String = "".to_string();
        let mut previous_first_letter: char = ' ';

        for word in sorted_words {
            let first_letter = word.chars().next().unwrap_or(' ').to_ascii_uppercase();
            if first_letter != previous_first_letter {
                previous_word = "".to_string();
            }

            let mut common_chars = 0;
            while common_chars < word.len()
                && common_chars < previous_word.len()
                && word.chars().nth(common_chars) == previous_word.chars().nth(common_chars)
            {
                common_chars += 1;
            }

            let word_number = number_by_word.get(&word).ok_or_else(|| {
                EncodingError::UnencodableData(format!("Can't find number for word \"{}\"", word))
            })?;

            let encoded_word = std::iter::once(common_chars as u8)
                .chain(encode_word(&word[common_chars..]))
                .chain(encode_uint16be(*word_number))
                .collect::<Vec<u8>>();

            let word_entries = words_entries_by_first_letter
                .get_mut(&first_letter)
                .ok_or_else(|| {
                    EncodingError::UnencodableData(format!(
                        "Word \"{}\" does not begin with a letter between A and Z",
                        word
                    ))
                })?;
            word_entries.push(encoded_word);

            previous_word = word.to_string();
            previous_first_letter = first_letter;
        }

        let mut header: [u8; 52] = [0; 52];
        let mut data: Vec<u8> = vec![];
        let mut offset = 52;
        for (index, letter) in LETTERS.iter().enumerate() {
            let Some(word_entries) = words_entries_by_first_letter.get(&letter) else {
                continue;
            };
            if word_entries.is_empty() {
                continue;
            }

            let offset_encoded = encode_uint16be(offset as u16);
            header[index * 2] = offset_encoded[0];
            header[index * 2 + 1] = offset_encoded[1];

            for word_entry in word_entries {
                data.extend_from_slice(word_entry);
                offset += word_entry.len();
            }
        }

        out.write_all(&header)?;
        out.write_all(&data)?;
        out.write_u8(0)?;
        Ok(())
    }
}

#[cfg(feature = "js")]
pub mod js {
    use wasm_bindgen::{JsValue, prelude::wasm_bindgen};

    use crate::{buffer::Buffer, resources::encode::Encode, word_list::WordList};

    #[wasm_bindgen(js_name = "buildWordsTok")]
    pub fn build_words_tok(object_list: &WordList) -> Result<Buffer, JsValue> {
        let encoded = object_list
            .encode_to_vec(())
            .map_err(|e| JsValue::from_str(format!("{:?}", e).as_str()))?;
        let buffer = Buffer::from(encoded);
        Ok(buffer)
    }
}

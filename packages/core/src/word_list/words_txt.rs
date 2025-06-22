use std::collections::HashMap;

use wasm_bindgen::prelude::wasm_bindgen;

use crate::word_list::{WordList, WordListEntry};

peg::parser! {
    grammar words_txt_parser() for str {
        rule line_terminator() -> ()
            = "\n" / "\r"

        rule white_space() -> ()
            = " " / "\t" / line_terminator()

        rule decimal_digit() -> char
            = d:$(['0'..='9']) { d.chars().next().unwrap() }

        rule word_number() -> u16
            = digits:$(decimal_digit()+) {
                digits.parse().unwrap()
            }

        rule bare_word_start() -> char
            = c:$(['a'..='z' | 'A'..='Z' | '-' | '_']) { c.chars().next().unwrap() }

        rule bare_word_part() -> char
            = c:$(['a'..='z' | 'A'..='Z' | '0'..='9' | '-' | '_' | '.']) { c.chars().next().unwrap() }

        rule bare_word() -> String
            = start:bare_word_start() rest:bare_word_part()* {
                std::iter::once(start).chain(rest.into_iter())
                    .collect::<String>()
            }

        rule source_character() -> char
            = c:[_] { c }

        rule single_escape_character() -> char
            = "\\" c:$(['"' | '\\' | '\'' | 'b' | 'f' | 'n' | 'r' | 't']) {
                let next_char = c.chars().next().unwrap();
                match next_char {
                    '"' | '\\' | '\'' => next_char,
                    'b' => '\u{0008}',
                    'f' => '\u{000C}',
                    'n' => '\n',
                    'r' => '\r',
                    't' => '\t',
                    'v' => '\u{000B}',
                    _ => unreachable!(),
                }
            }

        rule escape_character() -> char
            = c:$("x" / "u" / decimal_digit() / single_escape_character()) { c.chars().next().unwrap() }

        rule hex_digit() -> char
            = c:$(['0'..='9' | 'a'..='f' | 'A'..='F']) { c.chars().next().unwrap() }

        rule hex_escape_sequence() -> char
            = "\\x" hex:$(hex_digit() hex_digit()) {
                char::from_u32(u32::from_str_radix(hex, 16).unwrap()).unwrap()
            }

        rule non_escape_character() -> char
            = !(escape_character() / line_terminator()) c:(source_character()) { c }

        rule character_escape_sequence() -> char
            = "\\" c:(single_escape_character() / non_escape_character()) { c }

        rule null_escape_sequence() -> char
            = "\\" "0" !decimal_digit() { '\u{0000}' }

        rule escape_sequence() -> Option<char>
            = c:(character_escape_sequence() / null_escape_sequence() / hex_escape_sequence()) { Some(c) }

        rule line_continuation() -> Option<char>
            = "\\" line_terminator() { None }

        rule single_string_source_character() -> Option<char>
            = !("'" / "\\") c:(source_character()) { Some(c) }

        rule double_string_source_character() -> Option<char>
            = !("\"" / "\\") c:(source_character()) { Some(c) }

        rule string_literal() -> String
            = "\"" chars:((double_string_source_character() / escape_sequence() / line_continuation())*) "\"" {
                chars.into_iter().filter_map(|c| c).collect()
            }
            / "'" chars:((single_string_source_character() / escape_sequence() / line_continuation())*) "'" {
                chars.into_iter().filter_map(|c| c).collect()
            }

        rule word() -> String
            = " "* word:(bare_word() / string_literal()) {
                word
            }

        rule synonym_list() -> Vec<String>
            = words:(word()*) { words.into_iter().collect() }

        rule word_declaration() -> (u16, WordListEntry)
            = number:word_number() ":" words:synonym_list() {
                (number, WordListEntry {
                    number,
                    canonical_word: words.first().cloned().unwrap_or_default(),
                    words: words.into_iter().collect(),
                })
            }

        pub rule word_declarations() -> HashMap<u16, WordListEntry>
            = declarations:(word_declaration() ** line_terminator()) {
                declarations.into_iter().collect()
            }
    }
}

fn format_word(word: &str) -> String {
    if word.chars().any(|c| match c {
        'a'..='z' | 'A'..='Z' | '0'..='9' | '-' => false,
        _ => true,
    }) {
        format!("\"{}\"", word.replace('"', "\\\""))
    } else {
        word.to_string()
    }
}

#[wasm_bindgen(js_name = exportWords)]
pub fn export_words(word_list: &WordList) -> String {
    let mut word_numbers = word_list.words.keys().copied().collect::<Vec<_>>();
    word_numbers.sort_unstable();

    word_numbers
        .iter()
        .map(|word_number| {
            let words = word_list.words.get(word_number).unwrap();
            let mut words = words
                .iter_words()
                .map(|word| format_word(word))
                .collect::<Vec<_>>();
            words.sort_unstable();
            format!("{}: {}", word_number, words.join(" "))
        })
        .collect::<Vec<_>>()
        .join("\n")
}

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(typescript_type = "WordListSyntaxError")]
    pub type WordListSyntaxError;

    #[wasm_bindgen(constructor)]
    pub fn new(message: String, line: usize, column: usize, offset: usize) -> WordListSyntaxError;
}

#[wasm_bindgen(js_name = parseWordList)]
pub fn parse_word_list(input: &str) -> Result<WordList, WordListSyntaxError> {
    match words_txt_parser::word_declarations(input) {
        Ok(declarations) => Ok(WordList {
            words: declarations,
        }),
        Err(err) => {
            let error_object = WordListSyntaxError::new(
                err.to_string(),
                err.location.line,
                err.location.column,
                err.location.offset,
            );

            Err(error_object)
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use crate::{resources::file_provider::FileProvider, test_data::contest2_template_dir};

    use super::*;

    #[test]
    fn test_parsing() {
        let words_txt_data = contest2_template_dir()
            .read_file_utf8("words.txt")
            .expect("Failed to read test words.txt file as UTF-8");
        let word_list = words_txt_parser::word_declarations(&words_txt_data)
            .expect("Failed to parse test word list");

        assert_eq!(word_list.len(), 44);
        assert_eq!(
            word_list.get(&14),
            Some(&WordListEntry {
                number: 14,
                canonical_word: "y".to_string(),
                words: HashSet::from(["y".to_string(), "yes".to_string()]),
            })
        );
        assert_eq!(
            word_list.get(&9999),
            Some(&WordListEntry {
                number: 9999,
                canonical_word: "rol".to_string(),
                words: HashSet::from(["rol".to_string()]),
            })
        );
    }
}

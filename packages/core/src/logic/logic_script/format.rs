use std::collections::HashMap;

#[cfg(feature = "js")]
use wasm_bindgen::prelude::wasm_bindgen;

use crate::{
    agi_version::AGIVersion,
    logic::{
        asm::codegen::AsmCodeGenerationContext,
        logic_script::{
            codegen::codegen::GenerateLogicScript,
            compile::{errors::CompilationError, preprocess::parse_logic_script_raw},
        },
    },
    word_list::WordList,
};

pub fn format_logic_script(source: &str) -> Result<String, CompilationError> {
    let program = parse_logic_script_raw(source, &AGIVersion::default_v3())?;

    let word_list = WordList::new();
    let messages = HashMap::new();
    let context = AsmCodeGenerationContext {
        word_list: &word_list,
        messages: &messages,
    };

    Ok(program
        .into_iter()
        .map(|statement_with_location| statement_with_location.value)
        .collect::<Vec<_>>()
        .generate_logic_script(&context, 0)?)
}

#[cfg(feature = "js")]
#[wasm_bindgen(js_name = "formatLogicScript")]
pub fn js_format_logic_script(source: &str) -> Result<String, String> {
    format_logic_script(source).map_err(|err| format!("{}", err))
}

#[cfg(test)]
mod tests {
    use crate::{logic::logic_script::format::format_logic_script, test_data::uriquest};
    use similar_asserts::assert_eq;

    #[test]
    fn smoke_test() {
        let project = uriquest();
        let original = project
            .file_provider
            .read_file_utf8("src/logic/0.agilogic")
            .unwrap();

        let formatted = format_logic_script(&original).unwrap();

        assert_eq!(original, formatted);
    }
}

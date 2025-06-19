use crate::logic::LogicInstruction;

pub mod codegen;
pub mod expressions;
pub mod literals;
pub mod operators;

#[derive(Debug)]
pub struct LogicLabel<'a> {
    pub address: u16,
    pub label: String,
    pub references: Vec<&'a LogicInstruction>,
}

#[cfg(feature = "js")]
pub mod js {
    use super::*;
    use wasm_bindgen::prelude::wasm_bindgen;

    #[wasm_bindgen(js_name = "LogicLabel")]
    pub struct OwnedLogicLabel {
        pub address: u16,
        #[wasm_bindgen(getter_with_clone)]
        pub label: String,
        #[wasm_bindgen(getter_with_clone)]
        pub references: Vec<LogicInstruction>,
    }

    #[wasm_bindgen(js_class = "LogicLabel")]
    impl OwnedLogicLabel {
        #[wasm_bindgen(constructor)]
        pub fn new(address: u16, label: String, references: Vec<LogicInstruction>) -> Self {
            Self {
                address,
                label,
                references,
            }
        }

        pub fn test_export() -> String {
            "This is a test export".to_string()
        }
    }

    impl<'a> From<LogicLabel<'a>> for OwnedLogicLabel {
        fn from(label: LogicLabel<'a>) -> Self {
            OwnedLogicLabel {
                address: label.address,
                label: label.label,
                references: label.references.into_iter().cloned().collect(),
            }
        }
    }

    impl OwnedLogicLabel {
        pub fn to_logic_label(&self) -> LogicLabel<'_> {
            LogicLabel {
                address: self.address,
                label: self.label.clone(),
                references: self.references.iter().collect(),
            }
        }
    }
}

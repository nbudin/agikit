use serde::{Deserialize, Serialize};
use strum_macros::{AsRefStr, EnumString};
use wasm_bindgen::{JsValue, prelude::wasm_bindgen};

pub mod decode;
pub mod dirs;
pub mod encode;
pub mod file_provider;
pub mod pack;
pub mod resource_collection;

#[derive(Debug, Clone, Copy, PartialEq, Eq, EnumString, AsRefStr, Serialize, Deserialize, Hash)]
pub enum ResourceType {
    LOGIC,
    PIC,
    VIEW,
    SOUND,
}

impl Into<JsValue> for ResourceType {
    fn into(self) -> JsValue {
        JsValue::from_str(self.as_ref())
    }
}

impl TryFrom<JsValue> for ResourceType {
    type Error = strum::ParseError;

    fn try_from(value: JsValue) -> Result<Self, Self::Error> {
        let Some(str_value) = value.as_string() else {
            return Err(strum::ParseError::VariantNotFound);
        };

        ResourceType::try_from(str_value.as_str())
    }
}

pub type ResourceNumber = u16;

#[wasm_bindgen(typescript_custom_section)]
const TS_APPEND_CONTENT: &'static str = r#"
export enum ResourceType {
  LOGIC = 'LOGIC',
  PIC = 'PIC',
  VIEW = 'VIEW',
  SOUND = 'SOUND',
}

export type ResourceNumber = number;
"#;

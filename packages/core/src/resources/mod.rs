use serde::{Deserialize, Serialize};
use strum_macros::{AsRefStr, Display, EnumString};
use wasm_bindgen::prelude::wasm_bindgen;

pub mod decode;
pub mod dirs;
pub mod encode;
pub mod file_provider;
pub mod pack;
pub mod resource_collection;

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, EnumString, AsRefStr, Serialize, Deserialize, Hash, Display,
)]
#[wasm_bindgen(skip_typescript)]
pub enum ResourceType {
    LOGIC,
    PIC,
    VIEW,
    SOUND,
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

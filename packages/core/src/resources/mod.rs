use serde::{Deserialize, Serialize};
use strum_macros::{AsRefStr, EnumString};
use wasm_bindgen::prelude::wasm_bindgen;

use crate::{
    data_encoding::ReadHeterogeneousData,
    resources::{decode::Decode, encode::Encode},
};

pub mod decode;
pub mod dirs;
pub mod encode;
pub mod file_provider;
pub mod resource_collection;

#[derive(Debug, Clone, Copy, PartialEq, Eq, EnumString, AsRefStr, Serialize, Deserialize, Hash)]
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

pub trait Resource<'dec, Data: ReadHeterogeneousData, T: Encode + Decode<'dec, Data>> {
    fn resource_type(&self) -> ResourceType;
    fn resource_number(&self) -> ResourceNumber;
}

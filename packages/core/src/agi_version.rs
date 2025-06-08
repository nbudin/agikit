use std::fmt::Display;

use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};
use wasm_bindgen::prelude::wasm_bindgen;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Copy, Serialize_repr, Deserialize_repr)]
#[wasm_bindgen]
#[repr(u8)]
pub enum AGIMajorVersion {
    AGI2 = 2,
    AGI3 = 3,
}

impl From<u8> for AGIMajorVersion {
    fn from(value: u8) -> Self {
        match value {
            2 => AGIMajorVersion::AGI2,
            3 => AGIMajorVersion::AGI3,
            _ => panic!("Invalid AGI major version: {}", value),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Ord, Serialize, Deserialize)]
#[wasm_bindgen]
pub struct AGIVersion {
    pub major: AGIMajorVersion,
    pub minor: u32,
}

impl PartialOrd for AGIVersion {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        if self.major == other.major {
            self.minor.partial_cmp(&other.minor)
        } else {
            self.major.partial_cmp(&other.major)
        }
    }
}

#[wasm_bindgen]
impl AGIVersion {
    #[wasm_bindgen(constructor)]
    pub fn new(major: u8, minor: u32) -> Self {
        Self {
            major: AGIMajorVersion::from(major),
            minor,
        }
    }
}

impl Display for AGIVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.major {
            AGIMajorVersion::AGI2 => write!(f, "2.{:03}", self.minor),
            AGIMajorVersion::AGI3 => {
                let six_digit_minor = format!("{:06}", self.minor);
                let (minor, patch) = six_digit_minor.split_at(3);
                write!(f, "3.{}.{}", minor, patch)
            }
        }
    }
}

#[wasm_bindgen(js_name = "formatVersionNumber")]
pub fn format_version_number(version: &AGIVersion) -> String {
    format!("{}", version)
}

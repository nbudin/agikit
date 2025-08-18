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

impl AGIVersion {
    pub fn default_v2() -> Self {
        AGIVersion::new(2, 936)
    }

    pub fn default_v3() -> Self {
        AGIVersion::new(3, 2149)
    }

    pub fn detect_from_agidata_ovl(ovl_data: &[u8]) -> Option<Self> {
        for i in 0..(ovl_data.len() - "Version ".len()) {
            let version_tag = &ovl_data[i..i + ("Version ".len())];
            if version_tag == "Version ".as_bytes() {
                let version_index = i + "Version ".len();
                let major_num = ovl_data[version_index];
                let major = match major_num {
                    b'2' => AGIMajorVersion::AGI2,
                    b'3' => AGIMajorVersion::AGI3,
                    _ => {
                        continue;
                    }
                };
                if ovl_data[version_index + 1] != b'.' {
                    continue;
                }

                let Ok(minor1) =
                    String::from_utf8(ovl_data[version_index + 2..version_index + 5].to_vec())
                else {
                    continue;
                };
                if !minor1.chars().all(|c| '0' <= c && c <= '9') {
                    continue;
                }

                match major {
                    AGIMajorVersion::AGI2 => {
                        return Some(AGIVersion {
                            major,
                            minor: minor1.parse().unwrap(),
                        });
                    }
                    AGIMajorVersion::AGI3 => {
                        if ovl_data[version_index + 5] != b'.' {
                            continue;
                        }

                        let Ok(minor2) = String::from_utf8(
                            ovl_data[version_index + 6..version_index + 9].to_vec(),
                        ) else {
                            continue;
                        };
                        if !minor2.chars().all(|c| '0' <= c && c <= '9') {
                            continue;
                        }

                        return Some(AGIVersion {
                            major,
                            minor: format!("{minor1}{minor2}").parse().unwrap(),
                        });
                    }
                }
            }
        }

        None
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

#[cfg(feature = "js")]
mod js {
    use wasm_bindgen::prelude::wasm_bindgen;

    use crate::agi_version::AGIVersion;

    #[wasm_bindgen(js_name = "formatVersionNumber")]
    pub fn format_version_number(version: &AGIVersion) -> String {
        format!("{}", version)
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        agi_version::{AGIMajorVersion, AGIVersion},
        resources::file_provider::FileProvider,
        test_data::{contest2_template_dir, kq4demo},
    };

    #[test]
    pub fn test_detect_agiv2() {
        let ovl_data = contest2_template_dir()
            .read_file_bytes("AGIDATA.OVL")
            .unwrap();

        let agi_version = AGIVersion::detect_from_agidata_ovl(&ovl_data);
        assert_eq!(
            Some(AGIVersion {
                major: AGIMajorVersion::AGI2,
                minor: 917
            }),
            agi_version
        );
    }

    #[test]
    pub fn test_detect_agiv3() {
        let ovl_data = kq4demo().read_file_bytes("AGIDATA.OVL").unwrap();

        let agi_version = AGIVersion::detect_from_agidata_ovl(&ovl_data);
        assert_eq!(
            Some(AGIVersion {
                major: AGIMajorVersion::AGI3,
                minor: 2102
            }),
            agi_version
        );
    }
}

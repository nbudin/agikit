use std::collections::HashSet;

use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::wasm_bindgen;

use crate::{
    agi_version::{AGIMajorVersion, AGIVersion},
    resources::file_provider::FileProvider,
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[wasm_bindgen]
#[serde(rename_all = "camelCase")]
pub struct ProjectConfig {
    #[wasm_bindgen(getter_with_clone, js_name = "agiVersion")]
    pub agi_version: AGIVersion,
    #[wasm_bindgen(getter_with_clone, js_name = "gameId")]
    pub game_id: String,
}

#[wasm_bindgen]
impl ProjectConfig {
    #[wasm_bindgen(constructor)]
    pub fn new(agi_version: AGIVersion, game_id: String) -> Self {
        Self {
            agi_version,
            game_id,
        }
    }
}

impl ProjectConfig {
    pub fn detect_with_version<FP: FileProvider>(
        file_provider: &FP,
        version: AGIVersion,
    ) -> Option<Self> {
        match &version.major {
            AGIMajorVersion::AGI2 => Some(Self {
                agi_version: version,
                game_id: "AGI".to_string(),
            }),
            AGIMajorVersion::AGI3 => {
                let Ok(filenames) = file_provider.list_files(None) else {
                    return None;
                };
                let filenames = filenames
                    .iter()
                    .map(|filename| filename.to_uppercase())
                    .collect::<HashSet<_>>();

                let game_id = filenames.iter().find_map(|filename| {
                    if filename.ends_with("DIR") {
                        let game_id = &filename[0..(filename.len() - 3)];
                        if filenames.contains(format!("{}VOL.0", game_id).as_str()) {
                            Some(game_id)
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                });

                game_id.map(|game_id| Self {
                    agi_version: version,
                    game_id: game_id.to_string(),
                })
            }
        }
    }

    pub fn detect_from_filenames<FP: FileProvider>(file_provider: &FP) -> Option<Self> {
        if !file_provider.exists("WORDS.TOK") || !file_provider.exists("OBJECT") {
            return None;
        }

        if file_provider.exists("VOL.0")
            && file_provider.exists("LOGDIR")
            && file_provider.exists("PICDIR")
            && file_provider.exists("SNDDIR")
            && file_provider.exists("VIEWDIR")
        {
            Self::detect_with_version(file_provider, AGIVersion::default_v2())
        } else {
            Self::detect_with_version(file_provider, AGIVersion::default_v3())
        }
    }
}

impl Default for ProjectConfig {
    fn default() -> Self {
        Self {
            agi_version: AGIVersion::default_v2(),
            game_id: String::from("AGI"),
        }
    }
}

#[cfg(feature = "js")]
mod js {
    use wasm_bindgen::{JsValue, prelude::wasm_bindgen};
    use web_sys::js_sys::Uint8Array;

    use crate::{buffer::Buffer, project::config::ProjectConfig};

    #[wasm_bindgen(js_name = "getDefaultProjectConfig")]
    pub fn get_default_project_config() -> ProjectConfig {
        ProjectConfig::default()
    }

    #[wasm_bindgen(js_name = "readProjectConfig")]
    pub fn read_project_config(data: Buffer) -> Result<ProjectConfig, JsValue> {
        let data_array = Uint8Array::new(&data);
        let config: ProjectConfig = serde_json::from_slice(&data_array.to_vec())
            .map_err(|e| format!("Failed to read project config: {}", e))?;
        Ok(config)
    }

    #[wasm_bindgen(js_name = "encodeProjectConfig")]
    pub fn encode_project_config(config: ProjectConfig) -> Result<Buffer, JsValue> {
        let json = serde_json::to_string(&config)
            .map_err(|e| format!("Failed to write project config: {}", e))?;
        let data_array = Uint8Array::from(json.as_bytes());
        Ok(Buffer::from(data_array.buffer()))
    }
}

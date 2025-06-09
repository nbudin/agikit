use std::path::Path;

use serde::{Deserialize, Serialize};
use tsify::Tsify;
use wasm_bindgen::{prelude::wasm_bindgen, JsValue};
use web_sys::js_sys::Uint8Array;

use crate::{agi_version::AGIVersion, resource::ResourceType, wasm_utils::Buffer};

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

impl Default for ProjectConfig {
    fn default() -> Self {
        Self {
            agi_version: AGIVersion::new(2, 936),
            game_id: String::from("AGI"),
        }
    }
}

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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Tsify)]
#[serde(rename_all = "camelCase")]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub struct ExplicitVolumeResourceReference {
    pub resource_type: ResourceType,
    pub resource_number: u8,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub struct ExplicitVolumeSpecification {
    pub number: u8,
    pub resources: Vec<ExplicitVolumeResourceReference>,
}

impl From<ExplicitVolumeSpecificationFileSection> for ExplicitVolumeSpecification {
    fn from(section: ExplicitVolumeSpecificationFileSection) -> Self {
        let resources =
            section
                .resources
                .logic
                .into_iter()
                .map(|num| ExplicitVolumeResourceReference {
                    resource_type: ResourceType::LOGIC,
                    resource_number: num,
                })
                .chain(section.resources.view.into_iter().map(|num| {
                    ExplicitVolumeResourceReference {
                        resource_type: ResourceType::VIEW,
                        resource_number: num,
                    }
                }))
                .chain(section.resources.sound.into_iter().map(|num| {
                    ExplicitVolumeResourceReference {
                        resource_type: ResourceType::SOUND,
                        resource_number: num,
                    }
                }))
                .chain(section.resources.pic.into_iter().map(|num| {
                    ExplicitVolumeResourceReference {
                        resource_type: ResourceType::PIC,
                        resource_number: num,
                    }
                }))
                .collect();

        Self {
            number: section.number,
            resources,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub struct ExplicitVolumeSpecificationFileResourceList {
    pub logic: Vec<u8>,
    pub view: Vec<u8>,
    pub sound: Vec<u8>,
    pub pic: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExplicitVolumeSpecificationFileSection {
    pub number: u8,
    pub resources: ExplicitVolumeSpecificationFileResourceList,
}

impl From<ExplicitVolumeSpecification> for ExplicitVolumeSpecificationFileSection {
    fn from(spec: ExplicitVolumeSpecification) -> Self {
        let mut resources = ExplicitVolumeSpecificationFileResourceList {
            logic: Vec::new(),
            view: Vec::new(),
            sound: Vec::new(),
            pic: Vec::new(),
        };

        for resource in spec.resources {
            match resource.resource_type {
                ResourceType::LOGIC => resources.logic.push(resource.resource_number),
                ResourceType::VIEW => resources.view.push(resource.resource_number),
                ResourceType::SOUND => resources.sound.push(resource.resource_number),
                ResourceType::PIC => resources.pic.push(resource.resource_number),
            }
        }

        Self {
            number: spec.number,
            resources,
        }
    }
}

#[wasm_bindgen]
pub struct Project {
    #[wasm_bindgen(js_name = "basePath", getter_with_clone)]
    pub base_path: String,
    #[wasm_bindgen(getter_with_clone)]
    pub config: ProjectConfig,
}

#[wasm_bindgen]
impl Project {
    #[wasm_bindgen(constructor)]
    pub fn new(base_path: String, config: Option<ProjectConfig>) -> Self {
        let config = match config {
            Some(cfg) => cfg,
            None => {
                let config_path = Path::new(&base_path).join("agikit-project.json");
                if config_path.exists() {
                    let json = std::fs::read_to_string(&config_path)
                        .expect("Failed to read project config file");
                    serde_json::from_str(&json).expect("Error parsing project config")
                } else {
                    ProjectConfig::default()
                }
            }
        };

        Self { base_path, config }
    }

    #[wasm_bindgen(getter, js_name = "projectConfigPath")]
    pub fn project_config_path(&self) -> String {
        Path::new(&self.base_path)
            .join("agikit-project.json")
            .to_string_lossy()
            .to_string()
    }

    #[wasm_bindgen(getter, js_name = "sourcePath")]
    pub fn source_path(&self) -> String {
        Path::new(&self.base_path)
            .join("src")
            .to_string_lossy()
            .to_string()
    }

    #[wasm_bindgen(getter, js_name = "destinationPath")]
    pub fn destination_path(&self) -> String {
        Path::new(&self.base_path)
            .join("build")
            .to_string_lossy()
            .to_string()
    }

    #[wasm_bindgen(getter, js_name = "wordListSourcePath")]
    pub fn word_list_source_path(&self) -> String {
        Path::new(&self.base_path)
            .join("words.txt")
            .to_string_lossy()
            .to_string()
    }

    #[wasm_bindgen(getter, js_name = "objectListSourcePath")]
    pub fn object_list_source_path(&self) -> String {
        Path::new(&self.base_path)
            .join("object.json")
            .to_string_lossy()
            .to_string()
    }

    #[wasm_bindgen(getter, js_name = "explicitVolumeConfigPath")]
    pub fn explicit_volume_config_path(&self) -> String {
        Path::new(&self.base_path)
            .join("resourceVolumes.json")
            .to_string_lossy()
            .to_string()
    }

    #[wasm_bindgen(js_name = "readExplicitVolumeConfig")]
    pub fn read_explicit_volume_config(&self) -> Result<Vec<ExplicitVolumeSpecification>, JsValue> {
        let path = self.explicit_volume_config_path();
        if !Path::new(&path).exists() {
            return Ok(Vec::new());
        }

        let json = std::fs::read_to_string(&path)
            .map_err(|e| format!("Failed to read explicit volume config: {}", e))?;
        let sections: Vec<ExplicitVolumeSpecificationFileSection> = serde_json::from_str(&json)
            .map_err(|e| format!("Error parsing explicit volume config: {}", e))?;
        Ok(sections.into_iter().map(Into::into).collect())
    }
}

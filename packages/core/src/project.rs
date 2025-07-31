use std::{
    io::Cursor,
    path::{Path, PathBuf},
    str::FromStr,
    sync::{Arc, LazyLock},
};

use bitstream_io::{BigEndian, BitReader};
use serde::{Deserialize, Serialize};
use tsify::Tsify;
use wasm_bindgen::{JsValue, prelude::wasm_bindgen};
use web_sys::js_sys::Uint8Array;

use crate::{
    agi_version::{AGIMajorVersion, AGIVersion},
    buffer::Buffer,
    compression::bitstreams::DecodeBitstream,
    logic::LogicProgram,
    object_list::ObjectList,
    picture::Picture,
    resources::{
        ResourceType,
        decode::{Decode, DecodingError},
        dirs::{ResourceDirDecodeOptions, ResourceDirs},
        file_provider::{FileProvider, ReadSeek},
        resource_collection::{
            ResourceCollection, ResourceCollectionVersionData, ResourceReadResult,
        },
    },
    word_list::WordList,
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
    #[wasm_bindgen(skip)]
    pub file_provider: Arc<dyn FileProvider>,
    #[wasm_bindgen(skip)]
    resource_collection: LazyLock<
        ResourceCollection<Arc<dyn FileProvider>>,
        Box<dyn Fn() -> ResourceCollection<Arc<dyn FileProvider>>>,
    >,
    #[wasm_bindgen(getter_with_clone)]
    pub config: ProjectConfig,
}

impl Project {
    pub fn new<FP: FileProvider + 'static>(
        file_provider: FP,
        config: Option<ProjectConfig>,
    ) -> Self {
        let file_provider = Arc::new(file_provider) as Arc<dyn FileProvider>;

        let config = match config {
            Some(cfg) => cfg,
            None => {
                if file_provider.exists("agikit-project.json") {
                    let json = file_provider
                        .read_file_utf8("agikit-project.json")
                        .expect("Failed to read project config file");
                    serde_json::from_str(&json).expect("Error parsing project config")
                } else {
                    ProjectConfig::default()
                }
            }
        };

        let major_version_lazy = config.agi_version.major.clone();
        let file_provider_lazy = file_provider.clone();
        let game_id_lazy = config.game_id.clone();
        let init_resource_collection = move || {
            let decode_options = match major_version_lazy {
                AGIMajorVersion::AGI2 => ResourceDirDecodeOptions::AGI2 {
                    file_provider: file_provider_lazy.clone(),
                },
                AGIMajorVersion::AGI3 => ResourceDirDecodeOptions::AGI3 {
                    file_provider: file_provider_lazy.clone(),
                    game_id: game_id_lazy.clone(),
                },
            };

            let dirs = ResourceDirs::read(decode_options).unwrap();

            let version_data = match major_version_lazy {
                AGIMajorVersion::AGI2 => ResourceCollectionVersionData::AGI2,
                AGIMajorVersion::AGI3 => ResourceCollectionVersionData::AGI3(game_id_lazy.clone()),
            };

            ResourceCollection::new(version_data, file_provider_lazy.clone(), dirs)
        };

        Self {
            file_provider,
            config,
            resource_collection: LazyLock::new(Box::new(init_resource_collection)),
        }
    }

    pub fn resource_collection(&self) -> &ResourceCollection<Arc<dyn FileProvider>> {
        &self.resource_collection
    }

    pub fn read_resource_data(
        &self,
        resource_type: ResourceType,
        resource_number: u16,
    ) -> Result<ResourceReadResult, DecodingError> {
        self.resource_collection
            .read_resource_data(resource_type, resource_number)
    }

    pub fn decode_logic(&self, resource_number: u16) -> Result<LogicProgram, DecodingError> {
        let resource = self.read_resource_data(ResourceType::LOGIC, resource_number)?;
        LogicProgram::decode_from_bytes(&resource.data, &self.config.agi_version)
    }

    pub fn decode_picture(&self, resource_number: u16) -> Result<Picture, DecodingError> {
        let resource = self.read_resource_data(ResourceType::PIC, resource_number)?;
        let mut cursor = Cursor::new(resource.data);
        let mut reader = BitReader::endian(&mut cursor, BigEndian);
        Picture::decode_bitstream(&mut reader, resource.is_compressed_pic)
    }

    pub fn decode_object_list(&self) -> Result<ObjectList, DecodingError> {
        let mut data = self.file_provider.open_file("OBJECT")?;
        ObjectList::decode(&mut data, ())
    }

    pub fn decode_word_list(&self) -> Result<WordList, DecodingError> {
        let mut data = self.file_provider.open_file("WORDS.TOK")?;
        WordList::decode(&mut data, ())
    }
}

impl FileProvider for Project {
    fn base_path(&self) -> String {
        self.file_provider.base_path()
    }

    fn exists(&self, path: &str) -> bool {
        self.file_provider.exists(path)
    }

    fn open_file<'a>(&'a self, path: &str) -> Result<Box<dyn ReadSeek + 'a>, std::io::Error> {
        self.file_provider.open_file(path)
    }
}

#[wasm_bindgen]
impl Project {
    #[wasm_bindgen(constructor)]
    pub fn js_new(base_path: String, config: Option<ProjectConfig>) -> Self {
        let file_provider = PathBuf::from_str(&base_path).unwrap();
        Self::new(file_provider, config)
    }

    #[wasm_bindgen(getter, js_name = "basePath")]
    pub fn base_path(&self) -> String {
        self.file_provider.base_path()
    }

    #[wasm_bindgen(getter, js_name = "projectConfigPath")]
    pub fn project_config_path(&self) -> String {
        Path::new(&self.base_path())
            .join("agikit-project.json")
            .to_string_lossy()
            .to_string()
    }

    #[wasm_bindgen(getter, js_name = "sourcePath")]
    pub fn source_path(&self) -> String {
        Path::new(&self.base_path())
            .join("src")
            .to_string_lossy()
            .to_string()
    }

    #[wasm_bindgen(getter, js_name = "destinationPath")]
    pub fn destination_path(&self) -> String {
        Path::new(&self.base_path())
            .join("build")
            .to_string_lossy()
            .to_string()
    }

    #[wasm_bindgen(getter, js_name = "wordListSourcePath")]
    pub fn word_list_source_path(&self) -> String {
        Path::new(&self.base_path())
            .join("words.txt")
            .to_string_lossy()
            .to_string()
    }

    #[wasm_bindgen(getter, js_name = "objectListSourcePath")]
    pub fn object_list_source_path(&self) -> String {
        Path::new(&self.base_path())
            .join("object.json")
            .to_string_lossy()
            .to_string()
    }

    #[wasm_bindgen(getter, js_name = "explicitVolumeConfigPath")]
    pub fn explicit_volume_config_path(&self) -> String {
        Path::new(&self.base_path())
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

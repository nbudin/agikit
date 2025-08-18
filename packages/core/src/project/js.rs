use std::{collections::HashMap, path::PathBuf, str::FromStr, sync::Arc};

use super::Project;
use crate::{
    agi_version::AGIVersion,
    project::ProjectConfig,
    resources::{
        file_provider::FileProvider,
        pack::{
            EncodedResourceVolume, EncodedResourceVolumeCollection, ExplicitVolumeSpecification,
        },
    },
};
use tsify::serde_wasm_bindgen;
use wasm_bindgen::{JsValue, prelude::wasm_bindgen};
use web_sys::js_sys::Array;

#[wasm_bindgen(js_name = "Project")]
pub struct JsProject(Project<Arc<dyn FileProvider>>);

#[wasm_bindgen]
impl JsProject {
    #[wasm_bindgen(constructor)]
    pub fn new(base_path: String, config: Option<ProjectConfig>) -> Self {
        let file_provider = Arc::new(PathBuf::from_str(&base_path).unwrap());
        Self(Project::new(file_provider, config))
    }

    #[wasm_bindgen(getter)]
    pub fn config(&self) -> ProjectConfig {
        self.0.config.clone()
    }

    #[wasm_bindgen(getter, js_name = "basePath")]
    pub fn base_path(&self) -> String {
        self.0.base_path()
    }

    #[wasm_bindgen(getter, js_name = "projectConfigPath")]
    pub fn project_config_path(&self) -> String {
        self.0.project_config_path()
    }

    #[wasm_bindgen(getter, js_name = "sourcePath")]
    pub fn source_path(&self) -> String {
        self.0.source_path()
    }

    #[wasm_bindgen(getter, js_name = "destinationPath")]
    pub fn destination_path(&self) -> String {
        self.0.destination_path()
    }

    #[wasm_bindgen(getter, js_name = "wordListSourcePath")]
    pub fn word_list_source_path(&self) -> String {
        self.0.word_list_source_path()
    }

    #[wasm_bindgen(getter, js_name = "objectListSourcePath")]
    pub fn object_list_source_path(&self) -> String {
        self.0.object_list_source_path()
    }

    #[wasm_bindgen(getter, js_name = "explicitVolumeConfigPath")]
    pub fn explicit_volume_config_path(&self) -> String {
        self.0.explicit_volume_config_path()
    }

    #[wasm_bindgen(js_name = "readExplicitVolumeConfig")]
    pub fn read_explicit_volume_config(&self) -> Result<Vec<ExplicitVolumeSpecification>, JsValue> {
        self.0
            .read_explicit_volume_config()
            .map_err(|err| JsValue::from_str(format!("{}", err).as_str()))
    }
}

#[wasm_bindgen(js_name = "detectGame")]
pub fn detect_game(path: String) -> JsProject {
    let file_provider = Arc::new(PathBuf::from_str(&path).unwrap()) as Arc<dyn FileProvider>;
    JsProject(
        Project::detect(file_provider.clone()).unwrap_or_else(|| Project::new(file_provider, None)),
    )
}

fn volume_collection_from_js_resource_arrays(
    resource_volumes: Vec<JsValue>,
) -> Result<EncodedResourceVolumeCollection, JsValue> {
    let resource_volumes = resource_volumes
        .into_iter()
        .enumerate()
        .filter_map(|(volume_number, resource_array)| {
            if resource_array.is_null() || resource_array.is_undefined() {
                None
            } else {
                let array = Array::from(&resource_array);
                let encoded_resources = array
                    .into_iter()
                    .map(|encoded_resource| serde_wasm_bindgen::from_value(encoded_resource))
                    .collect::<Result<Vec<_>, _>>();

                Some(
                    encoded_resources.map(|encoded_resources| EncodedResourceVolume {
                        encoded_resources,
                        volume_number: volume_number as u8,
                    }),
                )
            }
        })
        .collect::<Result<Vec<_>, _>>()
        .map_err(|err| JsValue::from_str(format!("{}", err).as_str()))?;

    Ok(EncodedResourceVolumeCollection {
        volumes: resource_volumes
            .into_iter()
            .map(|vol| (vol.volume_number, vol))
            .collect(),
    })
}

#[wasm_bindgen(js_name = "writeV2ResourceFiles")]
pub fn write_v2_resource_files(
    output_path: String,
    resource_volumes: Vec<JsValue>,
    _logger: JsValue,
) -> Result<(), JsValue> {
    let project = Project::new(
        PathBuf::from_str(&output_path).unwrap(),
        Some(ProjectConfig {
            agi_version: AGIVersion::default_v2(),
            game_id: "AGI".to_string(),
        }),
    );

    let volume_collection = volume_collection_from_js_resource_arrays(resource_volumes)?;

    let dir_entries = volume_collection.build_dir_entries();
    project.resource_collection().lock().unwrap().dirs.dirs =
        dir_entries
            .into_iter()
            .fold(HashMap::new(), |mut dirs, entry| {
                let resource_dir = dirs.entry(entry.resource_type).or_default();
                resource_dir.insert(entry.resource_number, entry);
                dirs
            });

    project
        .write_volumes(&volume_collection)
        .and_then(|_| project.write_v2_dir_files())
        .map_err(|err| JsValue::from_str(format!("{}", err).as_str()))
}

#[wasm_bindgen(js_name = "writeV3ResourceFiles")]
pub fn write_v3_resource_files(
    output_path: String,
    game_id: String,
    resource_volumes: Vec<JsValue>,
    _logger: JsValue,
) -> Result<(), JsValue> {
    let project = Project::new(
        PathBuf::from_str(&output_path).unwrap(),
        Some(ProjectConfig {
            agi_version: AGIVersion::default_v3(),
            game_id,
        }),
    );

    let volume_collection = volume_collection_from_js_resource_arrays(resource_volumes)?;

    let dir_entries = volume_collection.build_dir_entries();
    project.resource_collection().lock().unwrap().dirs.dirs =
        dir_entries
            .into_iter()
            .fold(HashMap::new(), |mut dirs, entry| {
                let resource_dir = dirs.entry(entry.resource_type).or_default();
                resource_dir.insert(entry.resource_number, entry);
                dirs
            });

    project
        .write_volumes(&volume_collection)
        .and_then(|_| project.write_v3_dir_file())
        .map_err(|err| JsValue::from_str(format!("{}", err).as_str()))
}

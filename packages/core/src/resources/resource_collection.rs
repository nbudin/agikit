use std::{
    io::{Read, Seek, SeekFrom},
    path::Path,
};

use bitfield_struct::bitfield;
use wasm_bindgen::{prelude::wasm_bindgen, JsValue};
use web_sys::js_sys::Uint8Array;

use crate::{
    buffer::Buffer,
    compression::lzw::agi_lzw_decompress,
    data_encoding::ReadHeterogeneousData,
    resources::{
        decode::DecodingError,
        dirs::{DirEntry, ResourceDirs},
        file_provider::FileProvider,
        ResourceNumber, ResourceType,
    },
};

pub const RESOURCE_SIGNATURE: u16 = 0x1234;

pub fn read_v2_resource<P: FileProvider>(
    file_provider: &P,
    dir_entry: &DirEntry,
) -> Result<Vec<u8>, DecodingError> {
    let filename = format!("VOL.{}", dir_entry.volume_number);
    let mut file = file_provider.open_file(filename.as_str())?;
    file.seek(SeekFrom::Start(dir_entry.offset as u64))?;

    let signature = file.read_u16_be()?;
    if signature != RESOURCE_SIGNATURE {
        return Err(DecodingError::InvalidResourceSignature(signature));
    }

    let resource_vol_number = file.read_u8()?;
    if resource_vol_number != dir_entry.volume_number {
        return Err(DecodingError::VolumeNumberMismatch {
            expected: dir_entry.volume_number,
            actual: resource_vol_number,
        });
    }

    let length = file.read_u16_le()?;
    let mut data = vec![0; length as usize];
    file.read_exact(&mut data)?;
    Ok(data)
}

#[bitfield(u8)]
pub struct AGIV3ResourceVolNumberWithPicFlag {
    #[bits(7)]
    pub volume_number: u8,
    pub is_pic: bool,
}

pub fn read_v3_resource<P: FileProvider>(
    file_provider: &P,
    dir_entry: &DirEntry,
    game_id: &str,
) -> Result<Vec<u8>, DecodingError> {
    let filename = format!("{}VOL.{}", game_id, dir_entry.volume_number);
    let mut file = file_provider.open_file(filename.as_str())?;
    file.seek(SeekFrom::Start(dir_entry.offset as u64))?;

    let signature = file.read_u16_be()?;
    if signature != RESOURCE_SIGNATURE {
        return Err(DecodingError::InvalidResourceSignature(signature));
    }

    let resource_vol_number_with_pic_flag =
        AGIV3ResourceVolNumberWithPicFlag::from(file.read_u8()?);
    if resource_vol_number_with_pic_flag.volume_number() != dir_entry.volume_number {
        return Err(DecodingError::VolumeNumberMismatch {
            expected: dir_entry.volume_number,
            actual: resource_vol_number_with_pic_flag.volume_number(),
        });
    }

    let uncompressed_length = file.read_u16_le()?;
    let compressed_length = file.read_u16_le()?;

    let mut data = vec![0; compressed_length as usize];
    file.read_exact(&mut data)?;

    if resource_vol_number_with_pic_flag.is_pic() || uncompressed_length == compressed_length {
        Ok(data)
    } else {
        agi_lzw_decompress(&data).map_err(|e| e.into())
    }
}

#[wasm_bindgen]
pub struct JSReadResourceResult {
    #[wasm_bindgen(skip)]
    pub resource_type: ResourceType,
    pub number: ResourceNumber,
    #[wasm_bindgen(getter_with_clone)]
    pub data: Buffer,
}

#[wasm_bindgen]
impl JSReadResourceResult {
    #[wasm_bindgen(getter, js_name = "type")]
    pub fn resource_type(&self) -> String {
        self.resource_type.as_ref().to_string()
    }
}

#[wasm_bindgen(js_name = "readV2Resource")]
pub fn js_read_v2_resource(
    #[wasm_bindgen(js_name = "basePath")] base_path: String,
    #[wasm_bindgen(js_name = "dirEntry")] dir_entry: DirEntry,
) -> Result<JSReadResourceResult, JsValue> {
    let data = read_v2_resource(&Path::new(&base_path).to_path_buf(), &dir_entry)
        .map_err(|e| JsValue::from_str(&e.to_string()))?;
    let data_array = Uint8Array::new_with_length(data.len() as u32);
    data_array.copy_from(&data);
    let data_buffer = Buffer::from(data_array.buffer());
    Ok(JSReadResourceResult {
        resource_type: dir_entry.resource_type,
        number: dir_entry.resource_number,
        data: data_buffer,
    })
}

#[wasm_bindgen(js_name = "readV3Resource")]
pub fn js_read_v3_resource(
    #[wasm_bindgen(js_name = "basePath")] base_path: String,
    #[wasm_bindgen(js_name = "dirEntry")] dir_entry: DirEntry,
    #[wasm_bindgen(js_name = "gameId")] game_id: String,
) -> Result<JSReadResourceResult, JsValue> {
    let data = read_v3_resource(&Path::new(&base_path).to_path_buf(), &dir_entry, &game_id)
        .map_err(|e| JsValue::from_str(&e.to_string()))?;
    let data_array = Uint8Array::new_with_length(data.len() as u32);
    data_array.copy_from(&data);
    let data_buffer = Buffer::from(data_array.buffer());
    Ok(JSReadResourceResult {
        resource_type: dir_entry.resource_type,
        number: dir_entry.resource_number,
        data: data_buffer,
    })
}

pub enum ResourceCollectionVersionData {
    AGI2,
    AGI3(String),
}

pub struct ResourceCollection<P: FileProvider> {
    pub version_data: ResourceCollectionVersionData,
    pub file_provider: P,
    pub dirs: ResourceDirs,
}

impl<P: FileProvider> ResourceCollection<P> {
    pub fn new(
        version_data: ResourceCollectionVersionData,
        file_provider: P,
        dirs: ResourceDirs,
    ) -> Self {
        Self {
            version_data,
            file_provider,
            dirs,
        }
    }

    pub fn read_resource_data(
        &self,
        resource_type: ResourceType,
        resource_number: ResourceNumber,
    ) -> Result<Vec<u8>, DecodingError> {
        let Some(entry) = self.dirs.get_entry(resource_type, resource_number) else {
            return Err(DecodingError::ResourceNotFound {
                resource_type,
                resource_number,
            });
        };

        match &self.version_data {
            ResourceCollectionVersionData::AGI2 => read_v2_resource(&self.file_provider, entry),
            ResourceCollectionVersionData::AGI3(game_id) => {
                read_v3_resource(&self.file_provider, entry, game_id)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{resources::dirs::ResourceDirDecodeOptions, TEST_DATA_DIR};

    #[test]
    fn test_resource_collection_read_v2() {
        let file_provider = TEST_DATA_DIR.get_dir("uriquest").unwrap();
        let dirs = ResourceDirs::read(ResourceDirDecodeOptions::AGI2 { file_provider }).unwrap();

        let collection = ResourceCollection::new(
            ResourceCollectionVersionData::AGI2,
            file_provider.clone(),
            dirs,
        );

        let logic0 = collection
            .read_resource_data(ResourceType::LOGIC, 0)
            .expect("Failed to read logic resource 0");
        assert!(!logic0.is_empty(), "Logic resource 0 should not be empty");
    }

    #[test]
    fn test_resource_collection_read_v3() {
        let file_provider = TEST_DATA_DIR.get_dir("VTheGraphicalAdventureDemo").unwrap();
        let dirs = ResourceDirs::read(ResourceDirDecodeOptions::AGI3 {
            file_provider,
            game_id: "V".to_string(),
        })
        .unwrap();

        let collection = ResourceCollection::new(
            ResourceCollectionVersionData::AGI3("V".to_string()),
            file_provider.clone(),
            dirs,
        );

        let logic0 = collection
            .read_resource_data(ResourceType::LOGIC, 0)
            .expect("Failed to read logic resource 0");
        assert!(!logic0.is_empty(), "Logic resource 0 should not be empty");
    }
}

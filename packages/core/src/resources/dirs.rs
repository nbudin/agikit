use std::{
    collections::HashMap,
    io::{Cursor, Read, Seek, SeekFrom},
};

use wasm_bindgen::prelude::wasm_bindgen;

use crate::{
    data_encoding::ReadHeterogeneousData,
    resources::{
        decode::{Decode, DecodingError},
        file_provider::FileProvider,
        ResourceNumber, ResourceType,
    },
};

#[wasm_bindgen(skip_typescript)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DirEntry {
    #[wasm_bindgen(skip)]
    pub resource_type: ResourceType,
    #[wasm_bindgen(js_name = "resourceNumber")]
    pub resource_number: ResourceNumber,
    #[wasm_bindgen(js_name = "volumeNumber")]
    pub volume_number: u8,
    pub offset: u32,
}

impl Decode<'_> for Option<DirEntry> {
    type Options = (ResourceType, ResourceNumber);

    fn decode<'a, Data: ReadHeterogeneousData>(
        data: &'a mut Data,
        (resource_type, resource_number): Self::Options,
    ) -> Result<Self, DecodingError>
    where
        Self: Sized,
    {
        let vol_plus_high_order_nybble = data.read_u8()?;
        let mid_order_byte = data.read_u8()?;
        let low_order_byte = data.read_u8()?;

        let volume_number = vol_plus_high_order_nybble >> 4;
        let offset = ((vol_plus_high_order_nybble & 0x0F) as u32) << 16
            | ((mid_order_byte as u32) << 8)
            | low_order_byte as u32;

        if offset == 0xfffff && volume_number == 0x0f {
            Ok(None)
        } else {
            Ok(Some(DirEntry {
                resource_type,
                resource_number,
                volume_number,
                offset,
            }))
        }
    }
}

impl Decode<'_> for HashMap<ResourceNumber, DirEntry> {
    type Options = ResourceType;

    fn decode<'a, Data: ReadHeterogeneousData>(
        data: &'a mut Data,
        resource_type: Self::Options,
    ) -> Result<Self, DecodingError>
    where
        Self: Sized,
    {
        let mut entries = HashMap::new();
        let mut resource_number: ResourceNumber = 0;

        loop {
            let decode_result = Option::<DirEntry>::decode(data, (resource_type, resource_number));

            match decode_result {
                Ok(Some(dir_entry)) => {
                    entries.insert(resource_number, dir_entry);
                }
                Ok(None) => {}
                Err(err) => match err {
                    DecodingError::IoError(err) => {
                        if err.kind() == std::io::ErrorKind::UnexpectedEof {
                            break; // End of data reached
                        } else {
                            return Err(DecodingError::IoError(err));
                        }
                    }
                    _ => return Err(err),
                },
            }

            resource_number += 1;
        }

        Ok(entries)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResourceDirs {
    pub dirs: HashMap<ResourceType, HashMap<ResourceNumber, DirEntry>>,
}

pub enum ResourceDirDecodeOptions<P: FileProvider> {
    AGI2 { file_provider: P },
    AGI3 { file_provider: P, game_id: String },
}

impl ResourceDirs {
    pub fn get_entry(
        &self,
        resource_type: ResourceType,
        resource_number: ResourceNumber,
    ) -> Option<&DirEntry> {
        self.dirs
            .get(&resource_type)
            .and_then(|entries| entries.get(&resource_number))
    }

    pub fn read<P: FileProvider>(
        options: ResourceDirDecodeOptions<P>,
    ) -> Result<Self, DecodingError> {
        let mut dirs = HashMap::new();

        match options {
            ResourceDirDecodeOptions::AGI2 { file_provider } => {
                let mut read_dirfile = |filename: &str,
                                        resource_type: ResourceType|
                 -> Result<(), DecodingError> {
                    let mut dir_file = file_provider.open_file(filename)?;
                    let resource_dir =
                        HashMap::<ResourceNumber, DirEntry>::decode(&mut dir_file, resource_type)?;
                    dirs.insert(resource_type, resource_dir);
                    Ok(())
                };

                read_dirfile("LOGDIR", ResourceType::LOGIC)?;
                read_dirfile("VIEWDIR", ResourceType::VIEW)?;
                read_dirfile("PICDIR", ResourceType::PIC)?;
                read_dirfile("SNDDIR", ResourceType::SOUND)?;
            }
            ResourceDirDecodeOptions::AGI3 {
                file_provider,
                game_id,
            } => {
                let filename = format!("{}DIR", game_id);
                let mut file = file_provider.open_file(&filename)?;
                file.seek(SeekFrom::End(0))?;
                let file_end = file.stream_position()?;
                file.seek(SeekFrom::Start(0))?;

                let logic_start = file.read_u16_le()?;
                let pic_start = file.read_u16_le()?;
                let view_start = file.read_u16_le()?;
                let sound_start = file.read_u16_le()?;

                let mut read_section = |start: ResourceNumber,
                                        end: ResourceNumber,
                                        resource_type: ResourceType|
                 -> Result<(), DecodingError> {
                    file.seek(SeekFrom::Start(start as u64))?;
                    let mut buf = vec![0; (end - start) as usize];
                    file.read_exact(&mut buf.as_mut_slice())?;
                    dirs.insert(
                        resource_type,
                        HashMap::<u16, DirEntry>::decode(&mut Cursor::new(buf), resource_type)?,
                    );
                    Ok(())
                };

                read_section(logic_start, pic_start, ResourceType::LOGIC)?;
                read_section(pic_start, view_start, ResourceType::PIC)?;
                read_section(view_start, sound_start, ResourceType::VIEW)?;
                read_section(sound_start, file_end as u16, ResourceType::SOUND)?;
            }
        };

        Ok(Self { dirs })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        resources::ResourceType,
        test_data::{uriquest_dir, v_the_graphical_adventure_dir},
    };

    #[test]
    fn test_resource_dirs_read_v2() {
        let file_provider = uriquest_dir();
        let resource_dirs =
            ResourceDirs::read(ResourceDirDecodeOptions::AGI2 { file_provider }).unwrap();
        assert!(resource_dirs.dirs.contains_key(&ResourceType::LOGIC));
    }

    #[test]
    fn test_resource_dirs_read_v3() {
        let file_provider = v_the_graphical_adventure_dir();
        let resource_dirs = ResourceDirs::read(ResourceDirDecodeOptions::AGI3 {
            file_provider,
            game_id: "V".to_string(),
        })
        .unwrap();
        assert!(resource_dirs.dirs.contains_key(&ResourceType::LOGIC));
    }
}

#[cfg(feature = "js")]
pub mod js {
    use std::{collections::HashMap, fs::File, io::Cursor, str::FromStr};

    use wasm_bindgen::{prelude::wasm_bindgen, JsValue};
    use web_sys::js_sys::Uint8Array;

    use crate::{
        buffer::Buffer,
        resources::{
            decode::{Decode, DecodingError},
            dirs::DirEntry,
            ResourceNumber, ResourceType,
        },
    };

    #[wasm_bindgen]
    impl DirEntry {
        #[wasm_bindgen(constructor)]
        pub fn js_new(
            resource_type: &str,
            resource_number: ResourceNumber,
            volume_number: u8,
            offset: u32,
        ) -> Result<Self, JsValue> {
            let resource_type = ResourceType::from_str(resource_type)
                .map_err(|_| format!("Unknown resource type: {}", resource_type))?;
            Ok(DirEntry {
                resource_type,
                resource_number,
                volume_number,
                offset,
            })
        }

        #[wasm_bindgen(getter, js_name = "resourceType", skip_typescript)]
        pub fn js_resource_type(&self) -> String {
            self.resource_type.as_ref().to_string()
        }
    }

    #[wasm_bindgen(typescript_custom_section)]
    const DIR_ENTRY_TS_APPEND_CONTENT: &'static str = r#"
export class DirEntry {
  constructor(resourceType: string, resourceNumber: number, volumeNumber: number, offset: number);
  free(): void;
  resourceNumber: number;
  volumeNumber: number;
  offset: number;
  readonly resourceType: ResourceType;
}
"#;

    fn resource_dir_hashmap_to_js_optional_array(
        resources: HashMap<ResourceNumber, DirEntry>,
    ) -> Vec<JsValue> {
        let max_resource_number = resources.keys().max().cloned().unwrap_or(0);
        let mut entries = vec![JsValue::null(); (max_resource_number + 1) as usize];
        for (number, entry) in resources {
            entries[number as usize] = entry.into();
        }
        entries
    }

    #[wasm_bindgen(js_name = "readDirData", skip_typescript)]
    pub fn js_read_dir_data(
        #[wasm_bindgen(js_name = "dirData")] dir_data: Buffer,
        #[wasm_bindgen(js_name = "resourceType")] resource_type: &str,
    ) -> Result<Vec<JsValue>, JsValue> {
        let dir_data_vec = Uint8Array::new(&dir_data).to_vec();
        let mut cursor = Cursor::new(dir_data_vec);
        let resource_type = ResourceType::from_str(resource_type)
            .map_err(|_| format!("Unknown resource type: {}", resource_type))?;
        let resources = HashMap::<ResourceNumber, DirEntry>::decode(&mut cursor, resource_type)
            .map_err(|e| e.to_string())?;
        Ok(resource_dir_hashmap_to_js_optional_array(resources))
    }

    #[wasm_bindgen(typescript_custom_section)]
    const READ_DIR_DATA_APPEND_CONTENT: &'static str = r#"
export function readDirData(dirData: Buffer, resourceType: ResourceType): (DirEntry | undefined)[];
"#;

    #[wasm_bindgen(js_name = "readV2Dir", skip_typescript)]
    pub fn js_read_v2_dir(
        path: &str,
        #[wasm_bindgen(js_name = "resourceType")] resource_type: &str,
    ) -> Result<Vec<JsValue>, JsValue> {
        let mut file = File::open(path).map_err(|e| DecodingError::IoError(e).to_string())?;
        let resource_type = ResourceType::from_str(resource_type)
            .map_err(|_| format!("Unknown resource type: {}", resource_type))?;
        let resources = HashMap::<ResourceNumber, DirEntry>::decode(&mut file, resource_type)
            .map_err(|e| e.to_string())?;
        Ok(resource_dir_hashmap_to_js_optional_array(resources))
    }

    #[wasm_bindgen(typescript_custom_section)]
    const READ_V2_DIR_APPEND_CONTENT: &'static str = r#"
export function readV2Dir(path: string, resourceType: ResourceType): (DirEntry | undefined)[];
"#;
}

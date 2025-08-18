use std::{
    collections::HashMap,
    io::{Cursor, Read, Seek, SeekFrom},
};

use wasm_bindgen::prelude::wasm_bindgen;

use crate::{
    data_encoding::{ReadHeterogeneousData, WriteHeterogeneousData},
    resources::{
        ResourceNumber, ResourceType,
        decode::{Decode, DecodingError},
        encode::EncodingError,
        file_provider::FileProvider,
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
    pub fn from_dir_entries<I: IntoIterator<Item = DirEntry>>(entries: I) -> Self {
        let mut dirs: HashMap<ResourceType, HashMap<ResourceNumber, DirEntry>> = HashMap::new();
        for entry in entries {
            let type_dir = dirs
                .entry(entry.resource_type)
                .or_insert_with(|| HashMap::new());
            type_dir.insert(entry.resource_number, entry);
        }

        Self { dirs }
    }

    pub fn get_entry(
        &self,
        resource_type: ResourceType,
        resource_number: ResourceNumber,
    ) -> Option<&DirEntry> {
        self.dirs
            .get(&resource_type)
            .and_then(|entries| entries.get(&resource_number))
    }

    pub fn resource_numbers(
        &self,
        resource_type: ResourceType,
    ) -> impl Iterator<Item = ResourceNumber> {
        self.dirs
            .get(&resource_type)
            .map(|entries| entries.keys().copied())
            .unwrap_or_default()
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

    pub fn encode_v2_dir<Out: WriteHeterogeneousData>(
        &self,
        mut out: Out,
        resource_type: ResourceType,
    ) -> Result<(), EncodingError> {
        let dir = self.dirs.get(&resource_type);
        let max_resource_number = dir.and_then(|d| d.keys().max()).copied().unwrap_or(0);
        for resource_number in 0..=max_resource_number {
            let entry = dir.and_then(|d| d.get(&resource_number));
            match entry {
                Some(entry) => {
                    out.write_u8(
                        (((entry.volume_number as u32) << 4) + ((entry.offset & 0xf0000) >> 16))
                            as u8,
                    )?;
                    out.write_u8(((entry.offset & 0xff00) >> 8) as u8)?;
                    out.write_u8((entry.offset & 0xff) as u8)?;
                }
                None => {
                    out.write_u8(0xff)?;
                    out.write_u8(0xff)?;
                    out.write_u8(0xff)?;
                }
            }
        }

        Ok(())
    }

    pub fn encode_v3_dir<Out: WriteHeterogeneousData>(
        &self,
        mut out: Out,
    ) -> Result<(), EncodingError> {
        let blocks = [
            ResourceType::LOGIC,
            ResourceType::PIC,
            ResourceType::VIEW,
            ResourceType::SOUND,
        ]
        .into_iter()
        .map(|resource_type| {
            let mut block_data: Vec<u8> = vec![];
            self.encode_v2_dir(&mut Cursor::new(&mut block_data), resource_type)
                .map(|_| block_data)
        })
        .collect::<Result<Vec<_>, _>>()?;

        let mut offset: usize = 8;
        for block in blocks.iter() {
            out.write_u16_le(offset as u16)?;
            offset += block.len();
        }
        for block in blocks {
            out.write(&block)?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use similar_asserts::assert_eq;

    use super::*;
    use crate::{
        resources::ResourceType,
        test_data::{kq4demo_dir, uriquest_dir},
    };

    #[test]
    fn smoke_test_resource_dirs_v2() {
        let file_provider = uriquest_dir();
        let resource_dirs =
            ResourceDirs::read(ResourceDirDecodeOptions::AGI2 { file_provider }).unwrap();
        assert!(resource_dirs.dirs.contains_key(&ResourceType::LOGIC));

        let check_resource_type_dir = |resource_type, filename| {
            let mut reencoded: Vec<u8> = vec![];
            resource_dirs
                .encode_v2_dir(&mut Cursor::new(&mut reencoded), resource_type)
                .unwrap();
            let original = file_provider.read_file_bytes(filename).unwrap();
            assert_eq!(original, reencoded, "{} does not match", filename);
        };

        check_resource_type_dir(ResourceType::LOGIC, "LOGDIR");
        check_resource_type_dir(ResourceType::PIC, "PICDIR");
        check_resource_type_dir(ResourceType::VIEW, "VIEWDIR");
        check_resource_type_dir(ResourceType::SOUND, "SNDDIR");
    }

    #[test]
    fn smoke_test_resource_dirs_v3() {
        let file_provider = kq4demo_dir();
        let resource_dirs = ResourceDirs::read(ResourceDirDecodeOptions::AGI3 {
            file_provider,
            game_id: "DM".to_string(),
        })
        .unwrap();
        assert!(resource_dirs.dirs.contains_key(&ResourceType::LOGIC));

        let mut reencoded: Vec<u8> = vec![];
        resource_dirs
            .encode_v3_dir(&mut Cursor::new(&mut reencoded))
            .unwrap();
        let original = file_provider.read_file_bytes("DMDIR").unwrap();
        assert_eq!(original, reencoded);
    }
}

#[cfg(feature = "js")]
pub mod js {
    use std::{collections::HashMap, fs::File, io::Cursor, path::PathBuf, str::FromStr};

    use wasm_bindgen::{JsValue, convert::TryFromJsValue, prelude::wasm_bindgen};
    use web_sys::js_sys::Uint8Array;

    use crate::{
        agi_version::AGIVersion,
        buffer::Buffer,
        project::{Project, ProjectConfig},
        resources::{
            ResourceNumber, ResourceType,
            decode::{Decode, DecodingError},
            dirs::{DirEntry, ResourceDirDecodeOptions, ResourceDirs},
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

    fn js_optional_array_to_resource_dir_hashmap(
        resources: &[JsValue],
    ) -> Result<HashMap<ResourceNumber, DirEntry>, <DirEntry as TryFromJsValue>::Error> {
        resources
            .iter()
            .cloned()
            .filter_map(|resource| {
                if resource.is_null() || resource.is_undefined() {
                    None
                } else {
                    Some(
                        DirEntry::try_from_js_value(resource)
                            .map(|dir_entry| (dir_entry.resource_number, dir_entry)),
                    )
                }
            })
            .collect::<Result<HashMap<_, _>, _>>()
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

    #[wasm_bindgen(skip_typescript, js_name = "ResourceDir")]
    pub struct JsResourceDir {
        #[wasm_bindgen(js_name = "LOGIC", getter_with_clone)]
        pub logic: Vec<JsValue>,
        #[wasm_bindgen(js_name = "PIC", getter_with_clone)]
        pub pic: Vec<JsValue>,
        #[wasm_bindgen(js_name = "VIEW", getter_with_clone)]
        pub view: Vec<JsValue>,
        #[wasm_bindgen(js_name = "SOUND", getter_with_clone)]
        pub sound: Vec<JsValue>,
    }

    impl TryFrom<JsResourceDir> for ResourceDirs {
        type Error = <DirEntry as TryFromJsValue>::Error;
        fn try_from(js_dirs: JsResourceDir) -> Result<Self, Self::Error> {
            Ok(ResourceDirs::from_dir_entries(
                js_optional_array_to_resource_dir_hashmap(&js_dirs.logic)?
                    .values()
                    .chain(js_optional_array_to_resource_dir_hashmap(&js_dirs.pic)?.values())
                    .chain(js_optional_array_to_resource_dir_hashmap(&js_dirs.view)?.values())
                    .chain(js_optional_array_to_resource_dir_hashmap(&js_dirs.sound)?.values())
                    .cloned(),
            ))
        }
    }

    impl From<ResourceDirs> for JsResourceDir {
        fn from(dirs: ResourceDirs) -> Self {
            JsResourceDir {
                logic: resource_dir_hashmap_to_js_optional_array(
                    dirs.dirs
                        .get(&ResourceType::LOGIC)
                        .cloned()
                        .unwrap_or_default(),
                ),
                pic: resource_dir_hashmap_to_js_optional_array(
                    dirs.dirs
                        .get(&ResourceType::PIC)
                        .cloned()
                        .unwrap_or_default(),
                ),
                view: resource_dir_hashmap_to_js_optional_array(
                    dirs.dirs
                        .get(&ResourceType::VIEW)
                        .cloned()
                        .unwrap_or_default(),
                ),
                sound: resource_dir_hashmap_to_js_optional_array(
                    dirs.dirs
                        .get(&ResourceType::SOUND)
                        .cloned()
                        .unwrap_or_default(),
                ),
            }
        }
    }

    #[wasm_bindgen(js_name = "readV2ResourceDirs", skip_typescript)]
    pub fn js_read_v2_resource_dirs(game_path: &str) -> Result<JsResourceDir, JsValue> {
        let path = PathBuf::from(game_path);
        let dirs = ResourceDirs::read(ResourceDirDecodeOptions::AGI2 {
            file_provider: path,
        })
        .map_err(|err| JsValue::from_str(format!("{}", err).as_str()))?;

        Ok(dirs.into())
    }

    #[wasm_bindgen(js_name = "readV3ResourceDirs", skip_typescript)]
    pub fn js_read_v3_resource_dirs(
        game_path: &str,
        game_id: &str,
    ) -> Result<JsResourceDir, JsValue> {
        let path = PathBuf::from(game_path);
        let dirs = ResourceDirs::read(ResourceDirDecodeOptions::AGI3 {
            file_provider: path,
            game_id: game_id.to_string(),
        })
        .map_err(|err| JsValue::from_str(format!("{}", err).as_str()))?;

        Ok(dirs.into())
    }

    #[wasm_bindgen(typescript_custom_section)]
    const READ_RESOURCE_DIRS_APPEND_CONTENT: &'static str = r#"
export type ResourceDir = Record<ResourceType, (DirEntry | undefined)[]>;
export function readV2ResourceDirs(gamePath: string): ResourceDir;
export function readV3ResourceDir(gamePath: string, gameId: string): ResourceDir;
"#;

    #[wasm_bindgen(skip_typescript)]
    pub fn js_write_v2_dir(entries: Vec<JsValue>) -> Result<Buffer, JsValue> {
        let dir_entries: Vec<DirEntry> = entries
            .into_iter()
            .filter_map(|uncast_entry| {
                if uncast_entry.is_null() || uncast_entry.is_undefined() {
                    None
                } else {
                    Some(DirEntry::try_from_js_value(uncast_entry))
                }
            })
            .collect::<Result<Vec<_>, _>>()?;
        let resource_type = dir_entries.first().unwrap().resource_type;

        let resource_dir = ResourceDirs::from_dir_entries(dir_entries);
        let mut buf: Vec<u8> = vec![];
        resource_dir
            .encode_v2_dir(&mut Cursor::new(&mut buf), resource_type)
            .map_err(|err| JsValue::from_str(format!("{}", err).as_str()))?;
        Ok(Buffer::from(buf))
    }

    #[wasm_bindgen(typescript_custom_section)]
    const WRITE_RESOURCE_DIRS_APPEND_CONTENT: &'static str = r#"
export function writeV2Dir(entries: (DirEntry | undefined)[]): Buffer;
    "#;

    #[wasm_bindgen(js_name = "writeV2DirFiles")]
    pub fn js_write_v2_dir_files(
        output_path: String,
        resource_dir: JsResourceDir,
        _logger: JsValue,
    ) -> Result<(), JsValue> {
        let project = Project::new(
            PathBuf::from_str(&output_path).unwrap(),
            Some(ProjectConfig {
                agi_version: AGIVersion::default_v2(),
                game_id: "AGI".to_string(),
            }),
        );
        let collection_mutex = project.resource_collection();
        let mut resource_collection = collection_mutex.lock().unwrap();
        resource_collection.dirs = resource_dir.try_into()?;

        project
            .write_v2_dir_files()
            .map_err(|err| JsValue::from_str(format!("{}", err).as_str()))
    }

    #[wasm_bindgen(js_name = "writeV3DirFile")]
    pub fn js_write_v3_dir_file(
        output_path: String,
        game_id: String,
        resource_dir: JsResourceDir,
        _logger: JsValue,
    ) -> Result<(), JsValue> {
        let project = Project::new(
            PathBuf::from_str(&output_path).unwrap(),
            Some(ProjectConfig {
                agi_version: AGIVersion::new(3, 2149),
                game_id,
            }),
        );
        let collection_mutex = project.resource_collection();
        let mut resource_collection = collection_mutex.lock().unwrap();
        resource_collection.dirs = resource_dir.try_into()?;

        project
            .write_v3_dir_file()
            .map_err(|err| JsValue::from_str(format!("{}", err).as_str()))
    }
}

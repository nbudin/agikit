use std::{
    collections::{HashMap, HashSet},
    fmt::Display,
};

use serde::{Deserialize, Serialize};
use serde_with::serde_as;
use tsify::Tsify;
use wasm_bindgen::prelude::wasm_bindgen;

use crate::resources::{ResourceType, dirs::DirEntry, encode::Encode};

// pub const MAX_VOLUME_SIZE: usize = 0xfffff;

#[derive(Debug, Clone)]
pub enum PackingError {
    NotEnoughSpaceForResource(ResourceType, u8, usize),
}

impl Display for PackingError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PackingError::NotEnoughSpaceForResource(
                resource_type,
                resource_number,
                requested_size,
            ) => f.write_fmt(format_args!(
                "Not enough space for {} {resource_number} (requested {requested_size})",
                resource_type.as_ref()
            )),
        }
    }
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

#[derive(Debug, Clone, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
#[serde_as]
pub struct EncodedResource {
    pub resource_type: ResourceType,
    pub resource_number: u8,
    #[serde_as(as = "Base64")]
    pub data: Vec<u8>,
}

#[derive(Debug, Clone)]
pub struct EncodedResourceVolume {
    pub volume_number: u8,
    pub encoded_resources: Vec<EncodedResource>,
}

impl EncodedResourceVolume {
    pub fn build_dir_entries(&self) -> Vec<DirEntry> {
        let (_, dir_entries) = self.encoded_resources.iter().fold(
            (0, vec![]),
            |(offset, mut dir_entries), resource| {
                dir_entries.push(DirEntry {
                    resource_number: resource.resource_number as u16,
                    resource_type: resource.resource_type,
                    volume_number: self.volume_number,
                    offset,
                });
                (offset + resource.data.len() as u32, dir_entries)
            },
        );

        dir_entries
    }
}

impl Encode<'_> for EncodedResourceVolume {
    type Options = ();

    fn encode<Out: crate::data_encoding::WriteHeterogeneousData>(
        &self,
        mut out: Out,
        _options: Self::Options,
    ) -> Result<(), super::encode::EncodingError> {
        for encoded_resource in self.encoded_resources.iter() {
            out.write(&encoded_resource.data)?;
        }

        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct EncodedResourceVolumeCollection {
    pub volumes: HashMap<u8, EncodedResourceVolume>,
}

impl EncodedResourceVolumeCollection {
    pub fn build_dir_entries(&self) -> Vec<DirEntry> {
        self.volumes
            .values()
            .flat_map(|vol| vol.build_dir_entries())
            .collect()
    }
}

struct VolumePacker {
    max_volume_size: usize,
    volumes: HashMap<u8, EncodedResourceVolume>,
    volume_sizes: HashMap<u8, usize>,
    resources_by_type_and_number: HashMap<(ResourceType, u8), EncodedResource>,
    to_pack: HashSet<(ResourceType, u8)>,
}

impl VolumePacker {
    pub fn new(resources: Vec<EncodedResource>, max_volume_size: usize) -> Self {
        let resources_by_type_and_number: HashMap<_, _> = resources
            .into_iter()
            .map(|resource| ((resource.resource_type, resource.resource_number), resource))
            .collect();

        let to_pack = resources_by_type_and_number
            .keys()
            .cloned()
            .collect::<HashSet<_>>();

        Self {
            max_volume_size,
            volumes: HashMap::new(),
            volume_sizes: HashMap::new(),
            resources_by_type_and_number,
            to_pack,
        }
    }

    pub fn pack(
        mut self,
        explicit_volumes: Vec<ExplicitVolumeSpecification>,
    ) -> Result<EncodedResourceVolumeCollection, PackingError> {
        for explicit_volume in explicit_volumes.iter() {
            for resource_spec in explicit_volume.resources.iter() {
                self.pack_resource(
                    explicit_volume.number,
                    resource_spec.resource_type,
                    resource_spec.resource_number,
                )
            }
        }

        while let Some((resource_type, resource_number)) = self.to_pack.iter().cloned().next() {
            let data_len = self
                .resources_by_type_and_number
                .get(&(resource_type, resource_number))
                .unwrap()
                .data
                .len();

            let volume_with_space = (0..=255).find(|volume_number| {
                let volume_size = self.volume_sizes.entry(*volume_number).or_insert(0);
                *volume_size + data_len <= self.max_volume_size
            });

            let Some(volume_with_space) = volume_with_space else {
                return Err(PackingError::NotEnoughSpaceForResource(
                    resource_type,
                    resource_number,
                    data_len,
                ));
            };

            self.pack_resource(volume_with_space, resource_type, resource_number);
        }

        Ok(EncodedResourceVolumeCollection {
            volumes: self.volumes,
        })
    }

    fn pack_resource(
        &mut self,
        volume_number: u8,
        resource_type: ResourceType,
        resource_number: u8,
    ) {
        let volume = self
            .volumes
            .entry(volume_number)
            .or_insert_with(|| EncodedResourceVolume {
                volume_number: volume_number,
                encoded_resources: vec![],
            });
        let resource = self
            .resources_by_type_and_number
            .get(&(resource_type, resource_number))
            .unwrap()
            .clone();
        let volume_size = self.volume_sizes.entry(volume_number).or_insert(0);
        *volume_size += resource.data.len();
        volume.encoded_resources.push(resource);
        self.to_pack.remove(&(resource_type, resource_number));
    }
}

impl EncodedResourceVolumeCollection {
    pub fn pack_resources(
        resources: Vec<EncodedResource>,
        explicit_volumes: Vec<ExplicitVolumeSpecification>,
        max_volume_size: usize,
    ) -> Result<Self, PackingError> {
        let packer = VolumePacker::new(resources, max_volume_size);
        packer.pack(explicit_volumes)
    }
}

#[cfg(feature = "js")]
mod js {
    use wasm_bindgen::{JsValue, prelude::wasm_bindgen};

    use crate::{
        buffer::Buffer,
        resources::{
            encode::Encode,
            pack::{EncodedResource, EncodedResourceVolume},
        },
    };

    #[wasm_bindgen(js_name = "writeVolume")]
    pub fn write_volume(
        volume_number: u8,
        resources: Vec<EncodedResource>,
    ) -> Result<Vec<JsValue>, JsValue> {
        let volume = EncodedResourceVolume {
            volume_number,
            encoded_resources: resources,
        };

        let encoded = volume
            .encode_to_vec(())
            .map_err(|err| JsValue::from_str(format!("{}", err).as_str()))?;

        let dir_entries = volume.build_dir_entries();

        Ok(vec![Buffer::from(encoded).into(), dir_entries.into()])
    }
}

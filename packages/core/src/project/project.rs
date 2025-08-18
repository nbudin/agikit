use std::{
    io::Cursor,
    path::{Path, PathBuf},
    str::FromStr,
    sync::{Arc, LazyLock, Mutex},
};

use bitstream_io::{BigEndian, BitReader};

use crate::{
    agi_version::{AGIMajorVersion, AGIVersion},
    compression::bitstreams::DecodeBitstream,
    logic::LogicProgram,
    object_list::ObjectList,
    picture::Picture,
    project::config::ProjectConfig,
    resources::{
        ResourceType,
        decode::{Decode, DecodingError},
        dirs::{ResourceDirDecodeOptions, ResourceDirs},
        encode::{Encode, EncodingError},
        file_provider::{FileProvider, ReadSeek, WritableFileProvider, WriteSeek},
        pack::{
            EncodedResourceVolumeCollection, ExplicitVolumeSpecification,
            ExplicitVolumeSpecificationFileSection,
        },
        resource_collection::{
            ResourceCollection, ResourceCollectionVersionData, ResourceReadResult,
        },
    },
    word_list::WordList,
};

pub struct Project<FP: FileProvider> {
    pub file_provider: Arc<FP>,
    resource_collection: LazyLock<
        Arc<Mutex<ResourceCollection<Arc<FP>>>>,
        Box<dyn Fn() -> Arc<Mutex<ResourceCollection<Arc<FP>>>>>,
    >,
    pub config: ProjectConfig,
}

impl<FP: FileProvider> Project<FP> {
    pub fn detect(file_provider: FP) -> Option<Self>
    where
        FP: 'static,
    {
        if file_provider.exists("agikit-project.json") {
            let json = file_provider
                .read_file_utf8("agikit-project.json")
                .expect("Failed to read project config file");
            let config = serde_json::from_str(&json).expect("Error parsing project config");
            return Some(Self::new(file_provider, config));
        }

        if file_provider.exists("AGIDATA.OVL") {
            let ovl_data = file_provider
                .read_file_bytes("AGIDATA.OVL")
                .expect("Failed to read AGIDATA.OVL");

            if let Some(detected_version) = AGIVersion::detect_from_agidata_ovl(&ovl_data) {
                let config = ProjectConfig::detect_with_version(&file_provider, detected_version);
                return Some(Self::new(file_provider, config));
            }
        }

        ProjectConfig::detect_from_filenames(&file_provider)
            .map(|config| Self::new(file_provider, Some(config)))
    }

    pub fn new(file_provider: FP, config: Option<ProjectConfig>) -> Self
    where
        FP: 'static,
    {
        let file_provider = Arc::new(file_provider);

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

            Arc::new(Mutex::new(ResourceCollection::new(
                version_data,
                file_provider_lazy.clone(),
                dirs,
            )))
        };

        Self {
            file_provider,
            config,
            resource_collection: LazyLock::new(Box::new(init_resource_collection)),
        }
    }

    pub fn resource_collection(&self) -> Arc<Mutex<ResourceCollection<Arc<FP>>>> {
        self.resource_collection.clone()
    }

    pub fn project_config_path(&self) -> String {
        Path::new(&self.base_path())
            .join("agikit-project.json")
            .to_string_lossy()
            .to_string()
    }

    pub fn source_path(&self) -> String {
        Path::new(&self.base_path())
            .join("src")
            .to_string_lossy()
            .to_string()
    }

    pub fn destination_path(&self) -> String {
        Path::new(&self.base_path())
            .join("build")
            .to_string_lossy()
            .to_string()
    }

    pub fn word_list_source_path(&self) -> String {
        Path::new(&self.base_path())
            .join("words.txt")
            .to_string_lossy()
            .to_string()
    }

    pub fn object_list_source_path(&self) -> String {
        Path::new(&self.base_path())
            .join("object.json")
            .to_string_lossy()
            .to_string()
    }

    pub fn explicit_volume_config_path(&self) -> String {
        Path::new(&self.base_path())
            .join("resourceVolumes.json")
            .to_string_lossy()
            .to_string()
    }

    pub fn read_explicit_volume_config(
        &self,
    ) -> Result<Vec<ExplicitVolumeSpecification>, DecodingError> {
        let path = self.explicit_volume_config_path();
        if !Path::new(&path).exists() {
            return Ok(Vec::new());
        }

        let json = std::fs::read_to_string(&path)?;
        let sections: Vec<ExplicitVolumeSpecificationFileSection> = serde_json::from_str(&json)?;
        Ok(sections.into_iter().map(Into::into).collect())
    }

    pub fn read_resource_data(
        &self,
        resource_type: ResourceType,
        resource_number: u16,
    ) -> Result<ResourceReadResult, DecodingError> {
        self.resource_collection
            .lock()
            .unwrap()
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

impl<FP: FileProvider + WritableFileProvider> Project<FP> {
    pub fn write_v2_dir_files(&self) -> Result<(), EncodingError> {
        for (filename, resource_type) in [
            ("LOGDIR", ResourceType::LOGIC),
            ("PICDIR", ResourceType::PIC),
            ("SNDDIR", ResourceType::SOUND),
            ("VIEWDIR", ResourceType::VIEW),
        ] {
            let out = self.file_provider.create_file(
                PathBuf::from_str(&self.destination_path())
                    .unwrap()
                    .join(filename)
                    .as_os_str()
                    .to_str()
                    .unwrap(),
            )?;
            self.resource_collection
                .lock()
                .unwrap()
                .dirs
                .encode_v2_dir(out, resource_type)?;
        }

        Ok(())
    }

    pub fn write_v3_dir_file(&self) -> Result<(), EncodingError> {
        let out = self.file_provider.create_file(
            PathBuf::from_str(&self.destination_path())
                .unwrap()
                .join(format!("{}DIR", self.config.game_id).as_str())
                .as_os_str()
                .to_str()
                .unwrap(),
        )?;
        self.resource_collection
            .lock()
            .unwrap()
            .dirs
            .encode_v3_dir(out)?;
        Ok(())
    }

    pub fn write_dir_files(&self) -> Result<(), EncodingError> {
        match self.config.agi_version.major {
            AGIMajorVersion::AGI2 => self.write_v2_dir_files(),
            AGIMajorVersion::AGI3 => self.write_v3_dir_file(),
        }
    }

    pub fn write_object(&self, object_list: &ObjectList) -> Result<(), EncodingError> {
        let out = self.file_provider.create_file(
            PathBuf::from_str(&self.destination_path())
                .unwrap()
                .join("OBJECT")
                .as_os_str()
                .to_str()
                .unwrap(),
        )?;
        object_list.encode(out, ())?;
        Ok(())
    }

    pub fn write_words_tok(&self, word_list: &WordList) -> Result<(), EncodingError> {
        let out = self.file_provider.create_file(
            PathBuf::from_str(&self.destination_path())
                .unwrap()
                .join("WORDS.TOK")
                .as_os_str()
                .to_str()
                .unwrap(),
        )?;
        word_list.encode(out, ())?;
        Ok(())
    }

    pub fn write_volumes(
        &self,
        volumes: &EncodedResourceVolumeCollection,
    ) -> Result<(), EncodingError> {
        for (volume_number, volume) in volumes.volumes.iter() {
            let out = self.file_provider.create_file(
                PathBuf::from_str(&self.destination_path())
                    .unwrap()
                    .join(format!("VOL.{}", volume_number).as_str())
                    .as_os_str()
                    .to_str()
                    .unwrap(),
            )?;
            volume.encode(out, ())?;
        }

        Ok(())
    }
}

impl<FP: FileProvider> FileProvider for Project<FP> {
    fn base_path(&self) -> String {
        self.file_provider.base_path()
    }

    fn exists(&self, path: &str) -> bool {
        self.file_provider.exists(path)
    }

    fn open_file<'a>(&'a self, path: &str) -> Result<Box<dyn ReadSeek + 'a>, std::io::Error> {
        self.file_provider.open_file(path)
    }

    fn list_files(&self, path: Option<&str>) -> Result<Vec<String>, std::io::Error> {
        self.file_provider.list_files(path)
    }
}

impl<FP: FileProvider + WritableFileProvider> WritableFileProvider for Project<FP> {
    fn create_file<'a>(&'a self, path: &str) -> Result<Box<dyn WriteSeek + 'a>, std::io::Error> {
        self.file_provider.create_file(path)
    }

    fn create_dir_all(&self, path: &str) -> Result<(), std::io::Error> {
        self.file_provider.create_dir_all(path)
    }
}

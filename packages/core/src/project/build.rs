use std::{
    fmt::{Debug, Display},
    path::PathBuf,
    str::FromStr,
};

use log::{info, warn};

use crate::{
    agi_version::{AGIMajorVersion, AGIVersion},
    logic::logic_script::compile::{compile::compile_logic_script, errors::CompilationError},
    object_list::ObjectList,
    picture::Picture,
    project::Project,
    resources::{
        ResourceType,
        decode::DecodingError,
        dirs::ResourceDirs,
        encode::{Encode, EncodingError},
        file_provider::{FileProvider, WritableFileProvider},
        pack::{EncodedResource, EncodedResourceVolumeCollection, PackingError},
    },
    word_list::{WordList, words_txt},
};

pub enum BuildError {
    CompilationError(CompilationError),
    DecodingError(DecodingError),
    EncodingError(EncodingError),
    IoError(std::io::Error),
    PackingError(PackingError),
}

impl Display for BuildError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BuildError::CompilationError(error) => error.fmt(f),
            BuildError::DecodingError(error) => std::fmt::Display::fmt(error, f),
            BuildError::EncodingError(error) => std::fmt::Display::fmt(error, f),
            BuildError::IoError(error) => std::fmt::Display::fmt(error, f),
            BuildError::PackingError(error) => std::fmt::Display::fmt(error, f),
        }
    }
}

impl From<DecodingError> for BuildError {
    fn from(value: DecodingError) -> Self {
        Self::DecodingError(value)
    }
}

impl From<EncodingError> for BuildError {
    fn from(value: EncodingError) -> Self {
        Self::EncodingError(value)
    }
}

impl From<std::io::Error> for BuildError {
    fn from(value: std::io::Error) -> Self {
        Self::IoError(value)
    }
}

impl From<PackingError> for BuildError {
    fn from(value: PackingError) -> Self {
        Self::PackingError(value)
    }
}

impl From<CompilationError> for BuildError {
    fn from(value: CompilationError) -> Self {
        Self::CompilationError(value)
    }
}

pub struct ProjectBuildContext {
    pub agi_version: AGIVersion,
    pub object_list: ObjectList,
    pub word_list: WordList,
}

pub enum ProjectBuildResourceInput {
    RawResourceData {
        resource_type: ResourceType,
        resource_number: u8,
        path: String,
    },
    LogicScript {
        resource_number: u8,
        path: String,
    },
    PicJSON {
        resource_number: u8,
        path: String,
    },
}

impl ProjectBuildResourceInput {
    pub fn encode<FP: FileProvider>(
        &self,
        file_provider: &FP,
        context: &ProjectBuildContext,
    ) -> Result<EncodedResource, BuildError> {
        match self {
            ProjectBuildResourceInput::RawResourceData {
                resource_type,
                resource_number,
                path,
            } => {
                info!("Reading {}", path);
                Ok(EncodedResource {
                    data: file_provider.read_file_bytes(path.as_str())?,
                    resource_number: *resource_number,
                    resource_type: *resource_type,
                })
            }
            ProjectBuildResourceInput::LogicScript {
                resource_number,
                path,
            } => {
                info!("Compiling {}", path);
                let source_code = file_provider.read_file_utf8(path.as_str())?;
                let (program, diagnostics) = compile_logic_script(
                    source_code.as_str(),
                    &path,
                    &context.word_list,
                    &context.object_list,
                    &context.agi_version,
                    file_provider,
                )?;
                for diagnostic in diagnostics {
                    warn!("{}", diagnostic);
                }

                Ok(EncodedResource {
                    resource_type: ResourceType::LOGIC,
                    resource_number: *resource_number,
                    // TODO encrypt messages or not
                    data: program.encode_to_vec(false)?,
                })
            }
            ProjectBuildResourceInput::PicJSON {
                resource_number,
                path,
            } => {
                info!("Compiling {}", path);
                let picture: Picture = serde_json::from_reader(file_provider.open_file(&path)?)
                    .map_err(DecodingError::SerdeJsonError)?;
                Ok(EncodedResource {
                    resource_type: ResourceType::PIC,
                    resource_number: *resource_number,
                    data: picture
                        .encode_to_vec(context.agi_version.major == AGIMajorVersion::AGI3)?,
                })
            }
        }
    }
}

fn scan_resource_src_dir<FP: FileProvider>(
    file_provider: &FP,
    path: &str,
    extension: &str,
) -> Result<impl Iterator<Item = (String, u8)>, std::io::Error> {
    Ok(file_provider
        .list_files(Some(path))?
        .into_iter()
        .filter_map(move |filename| {
            if filename.ends_with(extension)
                && let Ok(resource_number) = filename.strip_suffix(extension).unwrap().parse::<u8>()
            {
                Some((
                    PathBuf::from_str(path)
                        .unwrap()
                        .join(filename)
                        .as_os_str()
                        .to_string_lossy()
                        .into_owned(),
                    resource_number,
                ))
            } else {
                None
            }
        }))
}

impl<FP: FileProvider + WritableFileProvider + 'static> Project<FP> {
    pub fn scan_build_resource_inputs(
        &self,
    ) -> Result<impl Iterator<Item = ProjectBuildResourceInput>, std::io::Error> {
        Ok(
            scan_resource_src_dir(&self.file_provider, "src/logic", ".agilogic")?
                .map(
                    |(path, resource_number)| ProjectBuildResourceInput::LogicScript {
                        resource_number,
                        path,
                    },
                )
                .chain(
                    scan_resource_src_dir(&self.file_provider, "src/pic", ".agipic")?.map(
                        |(path, resource_number)| ProjectBuildResourceInput::PicJSON {
                            resource_number,
                            path,
                        },
                    ),
                )
                .chain(
                    scan_resource_src_dir(&self.file_provider, "src/sound", ".agisound")?.map(
                        |(path, resource_number)| ProjectBuildResourceInput::RawResourceData {
                            resource_type: ResourceType::SOUND,
                            resource_number,
                            path,
                        },
                    ),
                )
                .chain(
                    scan_resource_src_dir(&self.file_provider, "src/view", ".agiview")?.map(
                        |(path, resource_number)| ProjectBuildResourceInput::RawResourceData {
                            resource_type: ResourceType::VIEW,
                            resource_number,
                            path,
                        },
                    ),
                ),
        )
    }

    pub fn build(&self) -> Result<(), BuildError> {
        let destination_path = self.destination_path();
        self.file_provider.create_dir_all(&destination_path)?;

        let word_list = words_txt::parse_word_list(
            self.file_provider
                .read_file_utf8(self.word_list_source_path().as_str())?
                .as_str(),
        )
        .map_err(DecodingError::WordListSyntaxError)?;
        let object_list: ObjectList = serde_json::from_reader(
            self.file_provider
                .open_file(self.object_list_source_path().as_str())?,
        )
        .map_err(DecodingError::SerdeJsonError)?;

        let context = ProjectBuildContext {
            agi_version: self.config.agi_version.clone(),
            object_list,
            word_list,
        };

        let encoded_resources = self
            .scan_build_resource_inputs()?
            .map(|input| input.encode(&self.file_provider, &context))
            .collect::<Result<Vec<_>, _>>()?;

        let volumes = EncodedResourceVolumeCollection::pack_resources(
            encoded_resources,
            self.read_explicit_volume_config()?,
            0xfffff,
        )?;

        let output_project = Project::new(self.file_provider.clone(), Some(self.config.clone()));
        output_project.resource_collection().lock().unwrap().dirs =
            ResourceDirs::from_dir_entries(volumes.build_dir_entries());
        output_project.write_dir_files()?;
        output_project.write_volumes(&volumes)?;

        info!("Writing WORDS.TOK");
        output_project.write_words_tok(&context.word_list)?;
        info!("Writing OBJECT");
        output_project.write_object(&context.object_list)?;

        Ok(())
    }
}

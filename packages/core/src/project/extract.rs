use std::{collections::HashSet, fmt::Display, path::Path};

use log::{info, warn};
use wasm_bindgen::prelude::wasm_bindgen;

use crate::{
    agi_version::AGIMajorVersion,
    logic::{
        asm::codegen::{AsmCodeGenerationError, generate_logic_asm},
        logic_script::codegen::{
            context::LogicScriptCodeGenerationContext, errors::LogicScriptCodeGenerationError,
            program_generator::LogicScriptProgramGenerator,
        },
    },
    object_list::ObjectList,
    project::Project,
    resources::{
        ResourceType,
        decode::{Decode, DecodingError},
        dirs::{DirEntry, ResourceDirDecodeOptions, ResourceDirs},
        encode::{Encode, EncodingError},
        file_provider::{FileProvider, WritableFileProvider},
    },
    word_list::{WordList, words_txt::export_words},
};

pub enum ExtractError {
    AsmCodeGenerationError(AsmCodeGenerationError),
    DecodingError(DecodingError),
    EncodingError(EncodingError),
    IoError(std::io::Error),
    LogicScriptCodeGenerationError(LogicScriptCodeGenerationError),
    SerdeJsonError(serde_json::Error),
}

impl From<DecodingError> for ExtractError {
    fn from(value: DecodingError) -> Self {
        ExtractError::DecodingError(value)
    }
}

impl From<EncodingError> for ExtractError {
    fn from(value: EncodingError) -> Self {
        ExtractError::EncodingError(value)
    }
}

impl From<std::io::Error> for ExtractError {
    fn from(value: std::io::Error) -> Self {
        ExtractError::IoError(value)
    }
}

impl From<LogicScriptCodeGenerationError> for ExtractError {
    fn from(value: LogicScriptCodeGenerationError) -> Self {
        ExtractError::LogicScriptCodeGenerationError(value)
    }
}

impl From<serde_json::Error> for ExtractError {
    fn from(value: serde_json::Error) -> Self {
        ExtractError::SerdeJsonError(value)
    }
}

impl From<AsmCodeGenerationError> for ExtractError {
    fn from(value: AsmCodeGenerationError) -> Self {
        ExtractError::AsmCodeGenerationError(value)
    }
}

impl Display for ExtractError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ExtractError::AsmCodeGenerationError(error) => error.fmt(f),
            ExtractError::DecodingError(error) => error.fmt(f),
            ExtractError::EncodingError(error) => error.fmt(f),
            ExtractError::IoError(error) => error.fmt(f),
            ExtractError::LogicScriptCodeGenerationError(error) => error.fmt(f),
            ExtractError::SerdeJsonError(error) => error.fmt(f),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[wasm_bindgen]
pub struct ResourceToExtract {
    pub resource_type: ResourceType,
    pub resource_number: u8,
}

#[wasm_bindgen]
pub struct ExtractConfig {
    pub decompiler_debug: bool,
    #[wasm_bindgen(getter_with_clone)]
    pub only_resources: Option<Vec<ResourceToExtract>>,
}

impl Default for ExtractConfig {
    fn default() -> Self {
        Self {
            decompiler_debug: false,
            only_resources: None,
        }
    }
}

impl<IFP: FileProvider> Project<IFP> {
    pub fn extract<OFP: FileProvider + WritableFileProvider>(
        &self,
        output_project: &mut Project<OFP>,
        options: Option<ExtractConfig>,
    ) -> Result<(), ExtractError> {
        let options = options.unwrap_or_default();

        let only_resources = options
            .only_resources
            .as_ref()
            .map(|resources| resources.iter().cloned().collect::<HashSet<_>>());

        info!(
            "Extracting {} to {}",
            self.base_path(),
            output_project.base_path()
        );
        info!("Using AGI version {}", self.config.agi_version);
        info!("Game ID: {}", self.config.game_id);

        let resource_dirs = match self.config.agi_version.major {
            AGIMajorVersion::AGI2 => ResourceDirs::read(ResourceDirDecodeOptions::AGI2 {
                file_provider: self.file_provider.clone(),
            }),
            AGIMajorVersion::AGI3 => ResourceDirs::read(ResourceDirDecodeOptions::AGI3 {
                file_provider: self.file_provider.clone(),
                game_id: self.config.game_id.clone(),
            }),
        }?;

        let mut warning_resources: Vec<DirEntry> = vec![];

        info!("Extracting WORDS.TOK");
        let word_list = WordList::decode(
            &mut self.open_file(
                &Path::new(&self.base_path())
                    .join("WORDS.TOK")
                    .to_string_lossy(),
            )?,
            (),
        )?;
        output_project
            .create_file(output_project.word_list_source_path().as_str())?
            .write(export_words(&word_list).as_bytes())?;

        info!("Extracting OBJECT");
        let object_list = ObjectList::decode(
            &mut self.open_file(
                &Path::new(&self.base_path())
                    .join("OBJECT")
                    .to_string_lossy(),
            )?,
            (),
        )?;
        output_project
            .create_file(output_project.object_list_source_path().as_str())?
            .write(serde_json::to_string_pretty(&object_list)?.as_bytes())?;

        for resource_type in &[
            ResourceType::LOGIC,
            ResourceType::PIC,
            ResourceType::SOUND,
            ResourceType::VIEW,
        ] {
            let Some(entries) = resource_dirs.dirs.get(resource_type) else {
                continue;
            };

            for (resource_number, entry) in entries {
                if let Some(ref only_resources) = only_resources
                    && !only_resources.contains(&ResourceToExtract {
                        resource_type: *resource_type,
                        resource_number: *resource_number as u8,
                    })
                {
                    continue;
                }

                info!(
                    "Extracting {resource_type} {resource_number} from volume {}",
                    entry.volume_number
                );

                match self.extract_resource(entry, output_project, &word_list, &options) {
                    Ok(_) => {}
                    Err(err) => {
                        warning_resources.push(entry.clone());
                        warn!("Couldn't extract {resource_type} {resource_number}: {err}");
                    }
                }
            }
        }

        info!("Writing project config");
        let config_file =
            output_project.create_file(output_project.project_config_path().as_str())?;
        serde_json::to_writer_pretty(config_file, &output_project.config)?;

        Ok(())
    }

    pub fn extract_resource<OFP: FileProvider + WritableFileProvider>(
        &self,
        dir_entry: &DirEntry,
        output_project: &mut Project<OFP>,
        word_list: &WordList,
        options: &ExtractConfig,
    ) -> Result<(), ExtractError> {
        let lowercase_resource_type = dir_entry.resource_type.to_string().to_lowercase();
        let dest_dir = Path::new(&output_project.source_path()).join(&lowercase_resource_type);
        let dest_path = dest_dir
            .join(format!(
                "{}.agi{}",
                dir_entry.resource_number, &lowercase_resource_type
            ))
            .to_string_lossy()
            .to_string();
        let mut output_file = output_project.create_file(&dest_path)?;

        match dir_entry.resource_type {
            ResourceType::LOGIC => {
                let logic = self.decode_logic(dir_entry.resource_number)?;

                if options.decompiler_debug {
                    let asm = generate_logic_asm(&logic, word_list, &[])?;
                    let mut asm_file = output_project.create_file(
                        &dest_dir
                            .join(format!("{}.agiasm", dir_entry.resource_number))
                            .to_string_lossy()
                            .to_string(),
                    )?;
                    asm_file.write_fmt(format_args!("{}", asm))?;
                }

                let context =
                    LogicScriptCodeGenerationContext::try_from_program(&logic, &word_list)?;

                #[cfg(feature = "dot")]
                if options.decompiler_debug {
                    let mut bbg_file = output_project.create_file(
                        &dest_dir
                            .join(format!("{}.basicBlockGraph.dot", dir_entry.resource_number))
                            .to_string_lossy()
                            .to_string(),
                    )?;
                    bbg_file.write_fmt(format_args!(
                        "{}",
                        context.basic_block_graph.to_dot(&context.asm_context)
                    ))?;

                    let mut dominator_file = output_project.create_file(
                        &dest_dir
                            .join(format!("{}.dominatorTree.dot", dir_entry.resource_number))
                            .to_string_lossy()
                            .to_string(),
                    )?;
                    dominator_file.write_fmt(format_args!(
                        "{}",
                        context.domination_analysis.dominators_to_dot(
                            &context.basic_block_graph.graph,
                            &|node_id, node_weight| {
                                node_weight.node_attrs(&context.asm_context, node_id)
                            }
                        )
                    ))?;

                    let mut post_dominator_file = output_project.create_file(
                        &dest_dir
                            .join(format!(
                                "{}.postDominatorTree.dot",
                                dir_entry.resource_number
                            ))
                            .to_string_lossy()
                            .to_string(),
                    )?;
                    post_dominator_file.write_fmt(format_args!(
                        "{}",
                        context.domination_analysis.post_dominators_to_dot(
                            &context.basic_block_graph.graph,
                            &|node_id, node_weight| {
                                node_weight.node_attrs(&context.asm_context, node_id)
                            }
                        )
                    ))?;
                }

                let generator = LogicScriptProgramGenerator::new(&context);
                let script = generator.generate_logic_script(&logic.messages)?;
                output_file.write_fmt(format_args!("{}", &script))?;
            }
            ResourceType::PIC => {
                let pic = self.decode_picture(dir_entry.resource_number)?;
                serde_json::to_writer_pretty(output_file, &pic)?;
            }
            ResourceType::VIEW => {
                let view = self.decode_view(dir_entry.resource_number)?;
                view.encode(output_file, ())?;
            }
            ResourceType::SOUND => {
                let sound = self.decode_ibmpcjr_sound(dir_entry.resource_number)?;
                sound.encode(output_file, ())?;
            }
        }

        Ok(())
    }
}

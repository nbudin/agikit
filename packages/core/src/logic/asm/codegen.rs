use std::{collections::HashMap, fmt::Display};

use crate::{
    logic::{
        asm::{
            expressions::{
                AsmLogicArgument, LogicArgument, LogicBooleanExpression, LogicIdentifier,
                ParsedLogicArgument,
            },
            literals::{LogicLiteral, LogicLiteralValue, StringLiteral},
            operators::LogicBooleanBinaryOperator,
            LogicLabel,
        },
        commands::AGICommand,
        LogicCommand, LogicCondition, LogicGoto, LogicInstruction, LogicMessages, LogicProgram,
    },
    word_list::WordList,
};

pub struct AsmCodeGenerationContext<'a> {
    pub logic: &'a LogicProgram,
    pub word_list: &'a WordList,
}

#[derive(Debug)]
pub enum AsmCodeGenerationError {
    UnlabeledJumpAddress(u16),
    TooManyArguments(AGICommand, usize),
    UnknownWord(u16),
    UnknownMessage(u16),
    SerdeJsonError(serde_json::Error),
    ErrorGeneratingInstruction(LogicInstruction, Box<AsmCodeGenerationError>),
}

impl Display for AsmCodeGenerationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AsmCodeGenerationError::UnlabeledJumpAddress(address) => {
                write!(f, "Unlabeled jump address: {}", address)
            }
            AsmCodeGenerationError::TooManyArguments(command, count) => write!(
                f,
                "Too many arguments for command {}: expected {}, got {}",
                command.name,
                command.arg_types.len(),
                count
            ),
            AsmCodeGenerationError::UnknownWord(word) => write!(f, "Unknown word: {}", word),
            AsmCodeGenerationError::UnknownMessage(index) => {
                write!(f, "Unknown message: {}", index)
            }
            AsmCodeGenerationError::SerdeJsonError(err) => err.fmt(f),
            AsmCodeGenerationError::ErrorGeneratingInstruction(instruction, err) => write!(
                f,
                "Error generating instruction at address {}: {}",
                instruction.address(),
                err
            ),
        }
    }
}

pub trait GenerateLogicAsm {
    fn generate_asm(
        &self,
        context: &AsmCodeGenerationContext,
        labels: &HashMap<u16, &LogicLabel>,
    ) -> Result<String, AsmCodeGenerationError>;
}

impl GenerateLogicAsm for LogicBooleanBinaryOperator {
    fn generate_asm(
        &self,
        _context: &AsmCodeGenerationContext,
        _labels: &HashMap<u16, &LogicLabel>,
    ) -> Result<String, AsmCodeGenerationError> {
        Ok(match self {
            LogicBooleanBinaryOperator::LessThan => "<".to_string(),
            LogicBooleanBinaryOperator::LessThanOrEqual => "<=".to_string(),
            LogicBooleanBinaryOperator::GreaterThan => ">".to_string(),
            LogicBooleanBinaryOperator::GreaterThanOrEqual => ">=".to_string(),
            LogicBooleanBinaryOperator::Equal => "==".to_string(),
            LogicBooleanBinaryOperator::NotEqual => "!=".to_string(),
        })
    }
}

impl GenerateLogicAsm for LogicLiteralValue {
    fn generate_asm(
        &self,
        _context: &AsmCodeGenerationContext,
        _labels: &HashMap<u16, &LogicLabel>,
    ) -> Result<String, AsmCodeGenerationError> {
        Ok(match self {
            LogicLiteralValue::Number(number) => format!("{}", number.value),
            LogicLiteralValue::String(string) => serde_json::to_string(&string.value())
                .map_err(|err| AsmCodeGenerationError::SerdeJsonError(err))?,
        })
    }
}

impl GenerateLogicAsm for LogicLiteral {
    fn generate_asm(
        &self,
        context: &AsmCodeGenerationContext,
        labels: &HashMap<u16, &LogicLabel>,
    ) -> Result<String, AsmCodeGenerationError> {
        self.value.generate_asm(context, labels)
    }
}

impl GenerateLogicAsm for LogicIdentifier {
    fn generate_asm(
        &self,
        _context: &AsmCodeGenerationContext,
        _labels: &HashMap<u16, &LogicLabel>,
    ) -> Result<String, AsmCodeGenerationError> {
        Ok(self.name.clone())
    }
}

impl GenerateLogicAsm for ParsedLogicArgument {
    fn generate_asm(
        &self,
        context: &AsmCodeGenerationContext,
        labels: &HashMap<u16, &LogicLabel>,
    ) -> Result<String, AsmCodeGenerationError> {
        match self {
            ParsedLogicArgument::Literal(literal) => literal.generate_asm(context, labels),
            ParsedLogicArgument::Identifier(identifier) => identifier.generate_asm(context, labels),
        }
    }
}

impl GenerateLogicAsm for LogicCommand {
    fn generate_asm(
        &self,
        context: &AsmCodeGenerationContext,
        labels: &HashMap<u16, &LogicLabel>,
    ) -> Result<String, AsmCodeGenerationError> {
        let command_name = &self.agi_command.name;
        let args = self
            .args
            .iter()
            .enumerate()
            .map(|(index, arg)| {
                Ok(AsmLogicArgument::new(
                    *arg as u16,
                    self.agi_command
                        .arg_types
                        .get(index)
                        .copied()
                        .ok_or_else(|| {
                            AsmCodeGenerationError::TooManyArguments(
                                self.agi_command.clone(),
                                self.args.len(),
                            )
                        })?,
                )
                .try_parse(context)?
                .generate_asm(context, labels)?)
            })
            .collect::<Result<Vec<_>, _>>()?;

        Ok(format!("{}({});", command_name, args.join(", ")))
    }
}

impl GenerateLogicAsm for LogicGoto {
    fn generate_asm(
        &self,
        _context: &AsmCodeGenerationContext,
        labels: &HashMap<u16, &LogicLabel>,
    ) -> Result<String, AsmCodeGenerationError> {
        let label = labels.get(&self.jump_address);
        match label {
            Some(label) => Ok(format!("goto {};", label.label)),
            None => Err(AsmCodeGenerationError::UnlabeledJumpAddress(
                self.jump_address,
            )),
        }
    }
}

impl<Arg: LogicArgument + GenerateLogicAsm> GenerateLogicAsm for LogicBooleanExpression<Arg> {
    fn generate_asm(
        &self,
        context: &AsmCodeGenerationContext,
        labels: &HashMap<u16, &LogicLabel>,
    ) -> Result<String, AsmCodeGenerationError> {
        let generate_sub_expression_asm =
            |sub_expression: &LogicBooleanExpression<Arg>| match sub_expression {
                LogicBooleanExpression::TestCall(_) | LogicBooleanExpression::NotExpression(_) => {
                    sub_expression.generate_asm(context, labels)
                }

                _ => Ok(format!(
                    "({})",
                    sub_expression.generate_asm(context, labels)?
                )),
            };

        match self {
            LogicBooleanExpression::TestCall(test_call) => {
                if test_call.test_name == "isset" && test_call.argument_list.len() == 1 {
                    let arg = &test_call.argument_list[0];
                    arg.generate_asm(context, labels)
                } else {
                    Ok(format!(
                        "{}({})",
                        test_call.test_name,
                        test_call
                            .argument_list
                            .iter()
                            .map(|arg| arg.generate_asm(context, labels))
                            .collect::<Result<Vec<_>, _>>()?
                            .join(", ")
                    ))
                }
            }
            LogicBooleanExpression::AndExpression(and_expression) => Ok(and_expression
                .clauses
                .iter()
                .map(generate_sub_expression_asm)
                .collect::<Result<Vec<_>, _>>()?
                .join(" && ")),
            LogicBooleanExpression::OrExpression(or_expression) => Ok(or_expression
                .clauses
                .iter()
                .map(generate_sub_expression_asm)
                .collect::<Result<Vec<_>, _>>()?
                .join(" || ")),
            LogicBooleanExpression::NotExpression(not_expression) => Ok(format!(
                "!{}",
                generate_sub_expression_asm(&not_expression.expression)?
            )),
            LogicBooleanExpression::BinaryOperation(operation) => Ok(format!(
                "{} {} {}",
                operation.left.generate_asm(context, labels)?,
                operation.operator.generate_asm(context, labels)?,
                operation.right.generate_asm(context, labels)?
            )),
            LogicBooleanExpression::Identifier(identifier) => Ok(identifier.name.clone()),
        }
    }
}

impl GenerateLogicAsm for LogicCondition {
    fn generate_asm(
        &self,
        context: &AsmCodeGenerationContext,
        labels: &HashMap<u16, &LogicLabel>,
    ) -> Result<String, AsmCodeGenerationError> {
        let Some(skip_label) = labels.get(&self.skip_address) else {
            return Err(AsmCodeGenerationError::UnlabeledJumpAddress(
                self.skip_address,
            ));
        };

        let boolean_expression = LogicBooleanExpression::from_clauses(&self.clauses, &context)?;
        let condition_asm = boolean_expression.generate_asm(context, labels)?;
        Ok(format!(
            "unless ({}) goto {};",
            condition_asm, skip_label.label
        ))
    }
}

impl GenerateLogicAsm for LogicInstruction {
    fn generate_asm(
        &self,
        context: &AsmCodeGenerationContext,
        labels: &HashMap<u16, &LogicLabel>,
    ) -> Result<String, AsmCodeGenerationError> {
        match self {
            LogicInstruction::Command(command) => command.generate_asm(context, labels),
            LogicInstruction::Condition(condition) => condition.generate_asm(context, labels),
            LogicInstruction::Goto(goto) => goto.generate_asm(context, labels),
        }
    }
}

pub fn generate_labels<'a>(
    instructions: &'a [LogicInstruction],
    existing_labels: &'a [LogicLabel],
) -> Vec<LogicLabel> {
    let mut target_addresses_with_refs: HashMap<u16, HashMap<u16, &'a LogicInstruction>> =
        HashMap::new();

    for instruction in instructions {
        match instruction {
            LogicInstruction::Goto(goto) => {
                let targets = target_addresses_with_refs
                    .entry(goto.jump_address)
                    .or_default();
                targets.entry(instruction.address()).or_insert(instruction);
            }
            LogicInstruction::Condition(condition) => {
                let targets = target_addresses_with_refs
                    .entry(condition.skip_address)
                    .or_default();
                targets.entry(instruction.address()).or_insert(instruction);
            }
            _ => {}
        }
    }

    for existing_label in existing_labels {
        target_addresses_with_refs.remove(&existing_label.address);
    }

    let mut generated_labels: Vec<_> = target_addresses_with_refs
        .into_iter()
        .map(|(address, _)| LogicLabel {
            address,
            label: format!("Address{}", address),
        })
        .collect();

    generated_labels.sort_unstable_by_key(|label| label.address);

    generated_labels
}

pub fn generate_logic_asm_instruction_with_possible_label(
    instruction: &LogicInstruction,
    labels: &HashMap<u16, &LogicLabel>,
    context: &AsmCodeGenerationContext,
) -> Result<String, AsmCodeGenerationError> {
    let line_label = labels.get(&instruction.address());
    let line_instruction = format!(
        "{} {}",
        instruction.address(),
        instruction.generate_asm(context, labels)?
    );

    match line_label {
        Some(label) => Ok(format!("\n{}:\n{}", label.label, line_instruction)),
        None => Ok(line_instruction),
    }
}

pub fn generate_logic_messages(messages: &LogicMessages) -> Result<String, AsmCodeGenerationError> {
    let mut sorted_message_keys = messages.keys().collect::<Vec<_>>();
    sorted_message_keys.sort_unstable();
    let message_lines = sorted_message_keys
        .iter()
        .map(|index| {
            let message = messages
                .get(*index)
                .ok_or_else(|| AsmCodeGenerationError::UnknownMessage(**index as u16))?;
            Ok(format!(
                "#message {} {}",
                *index + 1,
                serde_json::to_string(message)
                    .map_err(|err| AsmCodeGenerationError::SerdeJsonError(err))?
            ))
        })
        .collect::<Result<Vec<_>, _>>()?;

    Ok(std::iter::once("// messages")
        .chain(message_lines.iter().map(|line| line.as_str()))
        .collect::<Vec<_>>()
        .join("\n"))
}

pub fn generate_logic_asm(
    logic: &LogicProgram,
    word_list: &WordList,
    labels: &[LogicLabel],
) -> Result<String, AsmCodeGenerationError> {
    let labels_to_use = generate_labels(&logic.instructions, labels);
    let labels_by_address: HashMap<u16, &LogicLabel> = labels_to_use
        .iter()
        .map(|label| (label.address, label))
        .collect();
    let context = AsmCodeGenerationContext { logic, word_list };

    let asm_code = logic
        .instructions
        .iter()
        .map(|instruction| {
            generate_logic_asm_instruction_with_possible_label(
                instruction,
                &labels_by_address,
                &context,
            )
            .map_err(|err| {
                AsmCodeGenerationError::ErrorGeneratingInstruction(
                    instruction.clone(),
                    Box::new(err),
                )
            })
        })
        .collect::<Result<Vec<_>, _>>()?
        .join("\n");

    Ok(format!(
        "{}\n\n{}\n",
        asm_code,
        generate_logic_messages(&logic.messages)?
    ))
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use super::*;
    use crate::{
        agi_version::AGIVersion,
        resources::{
            decode::Decode,
            dirs::{ResourceDirDecodeOptions, ResourceDirs},
            file_provider::FileProvider,
            resource_collection::{ResourceCollection, ResourceCollectionVersionData},
            ResourceType,
        },
        test_data::uriquest_dir,
    };
    use pretty_assertions::assert_eq;

    #[test]
    fn smoke_test() {
        let file_provider = uriquest_dir();
        let dirs = ResourceDirs::read(ResourceDirDecodeOptions::AGI2 { file_provider }).unwrap();

        let collection = ResourceCollection::new(
            ResourceCollectionVersionData::AGI2,
            file_provider.clone(),
            dirs,
        );
        let logic_data = collection
            .read_resource_data(ResourceType::LOGIC, 0)
            .expect("Failed to read logic resource 0");
        let mut cursor = Cursor::new(logic_data);
        let logic_program = LogicProgram::decode(&mut cursor, &AGIVersion::new(2, 917))
            .expect("Failed to decode logic program");

        let words_tok_data = uriquest_dir()
            .read_file_bytes("WORDS.TOK")
            .expect("Failed to get WORDS.TOK file");
        let word_list = WordList::decode(&mut std::io::Cursor::new(words_tok_data), ())
            .expect("Failed to decode WORDS.TOK");

        let generated_asm = generate_logic_asm(&logic_program, &word_list, &[])
            .expect("Failed to generate ASM code");

        let expected_asm = uriquest_dir()
            .read_file_utf8("0.agiasm")
            .expect("Failed to read uriquest/0.agiasm file");

        assert_eq!(generated_asm, expected_asm);
    }
}

#[cfg(feature = "js")]
pub mod js {
    use std::collections::HashMap;

    use wasm_bindgen::{prelude::wasm_bindgen, JsValue};

    use crate::{
        logic::{
            asm::{
                codegen::{
                    generate_labels, generate_logic_asm,
                    generate_logic_asm_instruction_with_possible_label, generate_logic_messages,
                    AsmCodeGenerationContext, GenerateLogicAsm,
                },
                js::OwnedLogicLabel,
                LogicLabel,
            },
            LogicCommand, LogicInstruction, LogicProgram,
        },
        word_list::WordList,
    };

    #[wasm_bindgen(js_name = "CodeGenerationContext")]
    pub struct OwnedCodeGenerationContext {
        #[wasm_bindgen(getter_with_clone)]
        pub logic: LogicProgram,
        #[wasm_bindgen(getter_with_clone, js_name = "wordList")]
        pub word_list: WordList,
    }

    #[wasm_bindgen(js_class = "CodeGenerationContext")]
    impl OwnedCodeGenerationContext {
        #[wasm_bindgen(constructor)]
        pub fn new(logic: LogicProgram, word_list: WordList) -> Self {
            OwnedCodeGenerationContext { logic, word_list }
        }
    }

    impl OwnedCodeGenerationContext {
        pub fn to_asm_code_generation_context(&self) -> AsmCodeGenerationContext {
            super::AsmCodeGenerationContext {
                logic: &self.logic,
                word_list: &self.word_list,
            }
        }
    }

    #[wasm_bindgen(js_name = "generateLabels")]
    pub fn js_generate_labels(
        instructions: Vec<LogicInstruction>,
        #[wasm_bindgen(js_name = "existingLabels")] existing_labels: Option<Vec<OwnedLogicLabel>>,
    ) -> Vec<OwnedLogicLabel> {
        let existing_labels = existing_labels.unwrap_or_default();
        let existing_labels: Vec<LogicLabel> = existing_labels
            .iter()
            .map(|label| label.to_logic_label())
            .collect();
        let labels = generate_labels(&instructions, &existing_labels);
        labels.into_iter().map(OwnedLogicLabel::from).collect()
    }

    #[wasm_bindgen(js_name = "generateLogicCommandCode")]
    pub fn generate_logic_command_code(
        instruction: LogicCommand,
        context: &OwnedCodeGenerationContext,
    ) -> Result<String, JsValue> {
        instruction
            .generate_asm(&context.to_asm_code_generation_context(), &HashMap::new())
            .map_err(|err| JsValue::from_str(&format!("Error generating command code: {}", err)))
    }

    struct JsLabelMap {
        labels: HashMap<u16, LogicLabel>,
    }

    impl JsLabelMap {
        fn new(labels: &[OwnedLogicLabel]) -> Self {
            JsLabelMap {
                labels: labels
                    .iter()
                    .map(|label| (label.address, label.to_logic_label()))
                    .collect(),
            }
        }

        fn ref_map(&self) -> HashMap<u16, &LogicLabel> {
            self.labels.iter().map(|(k, v)| (*k, v)).collect()
        }
    }

    #[wasm_bindgen(js_name = "generateLogicAsmInstruction")]
    pub fn js_generate_logic_asm_instruction(
        instruction: &LogicInstruction,
        labels: Vec<OwnedLogicLabel>,
        context: &OwnedCodeGenerationContext,
    ) -> Result<String, JsValue> {
        let label_map = JsLabelMap::new(&labels);
        generate_logic_asm_instruction_with_possible_label(
            instruction,
            &label_map.ref_map(),
            &context.to_asm_code_generation_context(),
        )
        .map_err(|err| {
            JsValue::from_str(&format!(
                "Error generating instruction at address {}: {}",
                instruction.address(),
                err
            ))
        })
    }

    #[wasm_bindgen(js_name = "generateLogicAsmInstructionWithPossibleLabel")]
    pub fn js_generate_logic_asm_instruction_with_possible_label(
        instruction: &LogicInstruction,
        labels: Vec<OwnedLogicLabel>,
        context: &OwnedCodeGenerationContext,
    ) -> Result<String, JsValue> {
        let label_map = JsLabelMap::new(&labels);
        generate_logic_asm_instruction_with_possible_label(
            instruction,
            &label_map.ref_map(),
            &context.to_asm_code_generation_context(),
        )
        .map_err(|err| {
            JsValue::from_str(&format!(
                "Error generating instruction at address {}: {}",
                instruction.address(),
                err
            ))
        })
    }

    #[wasm_bindgen(js_name = "generateLogicMessages")]
    pub fn js_generate_logic_messages(logic: &LogicProgram) -> Result<String, JsValue> {
        generate_logic_messages(&logic.messages)
            .map_err(|err| JsValue::from_str(&format!("Error generating messages: {}", err)))
    }

    #[wasm_bindgen(js_name = "generateLogicAsm")]
    pub fn js_generate_logic_asm(
        logic: &LogicProgram,
        word_list: &WordList,
        labels: Option<Vec<OwnedLogicLabel>>,
    ) -> Result<String, JsValue> {
        let labels = labels.unwrap_or_default();
        let label_refs: Vec<_> = labels.iter().map(|label| label.to_logic_label()).collect();
        generate_logic_asm(logic, word_list, &label_refs)
            .map_err(|err| JsValue::from_str(&format!("Error generating ASM code: {}", err)))
    }
}

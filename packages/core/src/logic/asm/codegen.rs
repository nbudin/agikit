use std::collections::HashMap;

use crate::{
    logic::{
        asm::LogicLabel,
        commands::AGICommand,
        logic_script::{
            expressions::{
                LogicScriptArgument, LogicScriptBooleanExpression, LogicScriptIdentifier,
            },
            literals::{LogicScriptLiteral, LogicScriptLiteralValue},
            operators::LogicScriptBooleanBinaryOperator,
        },
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
    SerdeJsonError(serde_json::Error),
}

pub trait GenerateLogicAsm {
    fn generate_asm(
        &self,
        context: &AsmCodeGenerationContext,
        labels: &HashMap<u16, &LogicLabel>,
    ) -> Result<String, AsmCodeGenerationError>;
}

impl GenerateLogicAsm for LogicScriptBooleanBinaryOperator {
    fn generate_asm(
        &self,
        _context: &AsmCodeGenerationContext,
        _labels: &HashMap<u16, &LogicLabel>,
    ) -> Result<String, AsmCodeGenerationError> {
        Ok(match self {
            LogicScriptBooleanBinaryOperator::LessThan => "<".to_string(),
            LogicScriptBooleanBinaryOperator::LessThanOrEqual => "<=".to_string(),
            LogicScriptBooleanBinaryOperator::GreaterThan => ">".to_string(),
            LogicScriptBooleanBinaryOperator::GreaterThanOrEqual => ">=".to_string(),
            LogicScriptBooleanBinaryOperator::Equal => "==".to_string(),
            LogicScriptBooleanBinaryOperator::NotEqual => "!=".to_string(),
        })
    }
}

impl GenerateLogicAsm for LogicScriptLiteralValue {
    fn generate_asm(
        &self,
        _context: &AsmCodeGenerationContext,
        _labels: &HashMap<u16, &LogicLabel>,
    ) -> Result<String, AsmCodeGenerationError> {
        Ok(match self {
            LogicScriptLiteralValue::Number(number) => format!("{}", number.value),
            LogicScriptLiteralValue::String(string) => serde_json::to_string(&string.value())
                .map_err(|err| AsmCodeGenerationError::SerdeJsonError(err))?,
        })
    }
}

impl GenerateLogicAsm for LogicScriptLiteral {
    fn generate_asm(
        &self,
        context: &AsmCodeGenerationContext,
        labels: &HashMap<u16, &LogicLabel>,
    ) -> Result<String, AsmCodeGenerationError> {
        self.value.generate_asm(context, labels)
    }
}

impl GenerateLogicAsm for LogicScriptIdentifier {
    fn generate_asm(
        &self,
        _context: &AsmCodeGenerationContext,
        _labels: &HashMap<u16, &LogicLabel>,
    ) -> Result<String, AsmCodeGenerationError> {
        Ok(self.name.clone())
    }
}

impl GenerateLogicAsm for LogicScriptArgument {
    fn generate_asm(
        &self,
        context: &AsmCodeGenerationContext,
        labels: &HashMap<u16, &LogicLabel>,
    ) -> Result<String, AsmCodeGenerationError> {
        match self {
            LogicScriptArgument::Literal(literal) => literal.generate_asm(context, labels),
            LogicScriptArgument::Identifier(identifier) => identifier.generate_asm(context, labels),
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
                Ok(LogicScriptArgument::new(
                    *arg,
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
                    context,
                )?
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

impl GenerateLogicAsm for LogicScriptBooleanExpression {
    fn generate_asm(
        &self,
        context: &AsmCodeGenerationContext,
        labels: &HashMap<u16, &LogicLabel>,
    ) -> Result<String, AsmCodeGenerationError> {
        let generate_sub_expression_asm =
            |sub_expression: &LogicScriptBooleanExpression| match sub_expression {
                LogicScriptBooleanExpression::TestCall(_)
                | LogicScriptBooleanExpression::NotExpression(_) => {
                    sub_expression.generate_asm(context, labels)
                }

                _ => Ok(format!(
                    "({})",
                    sub_expression.generate_asm(context, labels)?
                )),
            };

        match self {
            LogicScriptBooleanExpression::TestCall(test_call) => {
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
            LogicScriptBooleanExpression::AndExpression(and_expression) => Ok(and_expression
                .clauses
                .iter()
                .map(generate_sub_expression_asm)
                .collect::<Result<Vec<_>, _>>()?
                .join(" && ")),
            LogicScriptBooleanExpression::OrExpression(or_expression) => Ok(or_expression
                .clauses
                .iter()
                .map(generate_sub_expression_asm)
                .collect::<Result<Vec<_>, _>>()?
                .join(" || ")),
            LogicScriptBooleanExpression::NotExpression(not_expression) => Ok(format!(
                "!{}",
                generate_sub_expression_asm(&not_expression.expression)?
            )),
            LogicScriptBooleanExpression::BinaryOperation(operation) => Ok(format!(
                "{} {} {}",
                operation.left.generate_asm(context, labels)?,
                operation.operator.generate_asm(context, labels)?,
                operation.right.generate_asm(context, labels)?
            )),
            LogicScriptBooleanExpression::Identifier(identifier) => Ok(identifier.name.clone()),
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

        let boolean_expression =
            LogicScriptBooleanExpression::from_clauses(&self.clauses, &context)?;
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
    existing_labels: &'a [LogicLabel<'a>],
) -> Vec<LogicLabel<'a>> {
    let mut target_addresses_with_refs: HashMap<u16, HashMap<u16, &'a LogicInstruction>> =
        HashMap::new();

    for instruction in instructions {
        match instruction {
            LogicInstruction::Goto(goto) => {
                let targets = target_addresses_with_refs.entry(goto.address).or_default();
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
        .map(|(address, references)| LogicLabel {
            address,
            label: format!("Address{}", address),
            references: references.into_values().collect(),
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
    let message_lines = messages
        .iter()
        .map(|(index, message)| {
            Ok(format!(
                "#message {} {}",
                index + 1,
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
    labels: &[LogicLabel<'_>],
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
        })
        .collect::<Result<Vec<_>, _>>()?
        .join("\n");

    Ok(format!(
        "{}\n\n{}",
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
            resource_collection::{ResourceCollection, ResourceCollectionVersionData},
            ResourceType,
        },
        TEST_DATA_DIR,
    };

    #[test]
    fn smoke_test() {
        let file_provider = TEST_DATA_DIR.get_dir("uriquest").unwrap();
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

        let words_tok_data = TEST_DATA_DIR
            .get_file("uriquest/WORDS.TOK")
            .expect("Failed to get WORDS.TOK file")
            .contents();
        let word_list = WordList::decode(&mut std::io::Cursor::new(words_tok_data), ())
            .expect("Failed to decode WORDS.TOK");

        let generated_asm = generate_logic_asm(&logic_program, &word_list, &[])
            .expect("Failed to generate ASM code");

        let expected_asm = TEST_DATA_DIR
            .get_file("uriquest/0.agiasm")
            .expect("Failed to get expected logic.asm file")
            .contents_utf8()
            .expect("Failed to read expected logic.asm file as UTF-8");
        assert_eq!(generated_asm, expected_asm);
    }
}

use std::collections::{HashSet, VecDeque};

use petgraph::graph::NodeIndex;

use crate::logic::{
    asm::expressions::{LogicBooleanExpression, LogicIdentifier, ParsedLogicArgument},
    logic_script::{
        basic_block_graph::{BasicBlock, BasicBlockControlFlow},
        codegen::{
            command_to_statement::CommandToStatement, context::LogicScriptCodeGenerationContext,
            errors::LogicScriptCodeGenerationError,
        },
        statements::{
            KeywordType, LogicScriptCommandCall, LogicScriptIfStatement, LogicScriptKeyword,
            LogicScriptLabel, LogicScriptStatement,
        },
    },
    LogicConditionClause,
};

pub struct LogicScriptProgramGenerator<'a> {
    context: &'a LogicScriptCodeGenerationContext<'a>,
    visited_basic_blocks: HashSet<NodeIndex>,
}

impl<'a> LogicScriptProgramGenerator<'a> {
    pub fn new(context: &'a LogicScriptCodeGenerationContext<'a>) -> Self {
        Self {
            context,
            visited_basic_blocks: HashSet::new(),
        }
    }

    pub fn generate_statements(
        mut self,
    ) -> Result<Vec<LogicScriptStatement<ParsedLogicArgument>>, LogicScriptCodeGenerationError>
    {
        let mut queue = VecDeque::from([self.context.basic_block_graph.root_block_id]);
        let mut statements = vec![];

        while let Some(block_id) = queue.pop_front() {
            if self.visited_basic_blocks.contains(&block_id) {
                continue;
            }

            statements.extend(self.generate_logic_script_for_basic_block(block_id, &mut queue)?);
        }

        // TODO: optimization passes

        Ok(statements)
    }

    fn generate_goto(&self, label: String) -> LogicScriptStatement<ParsedLogicArgument> {
        LogicScriptStatement::CommandCall(LogicScriptCommandCall {
            commmand_name: "goto".to_string(),
            argument_list: vec![ParsedLogicArgument::Identifier(LogicIdentifier {
                name: label,
            })],
        })
    }

    fn find_basic_block_label(
        &self,
        block_id: NodeIndex,
    ) -> Result<Option<String>, LogicScriptCodeGenerationError> {
        let block = self
            .context
            .get_block(block_id)
            .ok_or_else(|| LogicScriptCodeGenerationError::BlockNotFound(block_id))?;

        match block {
            &BasicBlock::SinglePath(ref block) => {
                if let Some(first_command) = block.commands.first() {
                    return Ok(first_command
                        .label
                        .as_ref()
                        .map(|label| label.label.clone()));
                }
            }
            BasicBlock::Conditional(_) => {}
        }

        Ok(block.label().map(|l| l.to_owned()))
    }

    fn generate_command_statements(
        &self,
        block_id: NodeIndex,
    ) -> Result<Vec<LogicScriptStatement<ParsedLogicArgument>>, LogicScriptCodeGenerationError>
    {
        let block = self
            .context
            .get_block(block_id)
            .ok_or_else(|| LogicScriptCodeGenerationError::BlockNotFound(block_id))?;
        let mut statements = Vec::new();

        if let Some(label) = self.find_basic_block_label(block_id)? {
            statements.push(LogicScriptStatement::Label(LogicScriptLabel { label }));
        }

        if let BasicBlock::SinglePath(block) = block {
            statements.extend(
                block
                    .commands
                    .iter()
                    .map(|command| command.command.to_statement(&self.context))
                    .collect::<Result<Vec<_>, _>>()?,
            );
        }

        Ok(statements)
    }

    fn generate_single_path_code(
        &mut self,
        block_id: NodeIndex,
        next_block_id: Option<NodeIndex>,
        queue: &mut VecDeque<NodeIndex>,
    ) -> Result<Vec<LogicScriptStatement<ParsedLogicArgument>>, LogicScriptCodeGenerationError>
    {
        let command_statements = self.generate_command_statements(block_id)?;

        if let Some(next_block_id) = next_block_id {
            if self.context.dominates(block_id, next_block_id)
                && self.context.post_dominates(next_block_id, block_id)
            {
                let next_block_statements =
                    self.generate_logic_script_for_basic_block(next_block_id, queue)?;
                return Ok(command_statements
                    .into_iter()
                    .chain(next_block_statements.into_iter())
                    .collect::<Vec<_>>());
            } else {
                let next_block_label = self.find_basic_block_label(next_block_id)?;
                if let Some(next_block_label) = next_block_label {
                    queue.push_back(next_block_id);
                    if self.visited_basic_blocks.contains(&next_block_id) {
                        return Ok(command_statements
                            .into_iter()
                            .chain(std::iter::once(self.generate_goto(next_block_label)))
                            .collect());
                    }
                }
            }
        }

        Ok(command_statements)
    }

    fn generate_conditional_code(
        &mut self,
        block_id: NodeIndex,
        conditions: &[LogicConditionClause],
        then_id: NodeIndex,
        else_id: Option<NodeIndex>,
        queue: &mut VecDeque<NodeIndex>,
    ) -> Result<Vec<LogicScriptStatement<ParsedLogicArgument>>, LogicScriptCodeGenerationError>
    {
        let mut generate_branch_code = |block_id: NodeIndex, to_block_id: Option<NodeIndex>| {
            if let Some(to_block_id) = to_block_id {
                if !self.context.dominates(block_id, to_block_id) {
                    let Some(label) = self.find_basic_block_label(to_block_id)? else {
                        return Err(LogicScriptCodeGenerationError::ConditionalToUnlabeledBlock(
                            to_block_id,
                            self.context.get_block(to_block_id).cloned(),
                        ));
                    };

                    return Ok((
                        vec![self.generate_goto(label)],
                        VecDeque::from([to_block_id]),
                    ));
                }
            }

            let mut branch_queue = VecDeque::new();
            let mut branch_code = to_block_id
                .map(|to_block_id| {
                    self.generate_logic_script_for_basic_block(to_block_id, &mut branch_queue)
                })
                .unwrap_or(Ok(vec![]))?;

            if let Some(to_block_id) = to_block_id {
                if branch_queue.len() > 0
                    && !self.context.post_dominates(branch_queue[0], to_block_id)
                {
                    let Some(label) = self.find_basic_block_label(branch_queue[0])? else {
                        return Err(LogicScriptCodeGenerationError::ConditionalToUnlabeledBlock(
                            branch_queue[0],
                            self.context.get_block(branch_queue[0]).cloned(),
                        ));
                    };

                    branch_code.push(self.generate_goto(label));
                }
            }

            Ok((branch_code, branch_queue))
        };

        let (then_statements, mut then_queue) = generate_branch_code(block_id, Some(then_id))?;
        let (else_statements, else_queue) = generate_branch_code(block_id, else_id)?;

        let mut if_statement = LogicScriptIfStatement {
            conditions: LogicBooleanExpression::from_clauses(
                conditions,
                &self.context.asm_context,
            )?,
            if_keyword: LogicScriptKeyword {
                keyword: KeywordType::If,
            },
            else_keyword: else_id.map(|_| LogicScriptKeyword {
                keyword: KeywordType::Else,
            }),
            then_statements: then_statements.into_iter().map(Box::new).collect(),
            else_statements: else_statements.into_iter().map(Box::new).collect(),
        };

        let mut subsequent_code = vec![];
        if let Some(else_id) = else_id {
            if self.context.post_dominates(else_id, block_id)
                && then_queue
                    .iter()
                    .all(|next_block_id| self.context.dominates(else_id, *next_block_id))
            {
                // else clause can be unrolled
                subsequent_code.extend_from_slice(
                    &if_statement
                        .else_statements
                        .iter()
                        .map(|stmt| stmt.as_ref().clone())
                        .collect::<Vec<_>>(),
                );
                if_statement.else_statements.clear();
                then_queue.clear();
            }
        }

        let mut inner_queue: VecDeque<_> = then_queue
            .into_iter()
            .chain(else_queue.into_iter())
            .collect();

        while let Some(inner_block_id) = inner_queue.pop_front() {
            if self.visited_basic_blocks.contains(&inner_block_id) {
                continue;
            }

            if !self.context.dominates(block_id, inner_block_id) {
                queue.push_back(inner_block_id);
                continue;
            }

            let mut to_add = VecDeque::new();
            let inner_code =
                self.generate_logic_script_for_basic_block(inner_block_id, &mut to_add)?;
            inner_queue.extend(to_add);
            subsequent_code.extend(inner_code.into_iter());
        }

        Ok(self
            .generate_command_statements(block_id)?
            .into_iter()
            .chain(std::iter::once(LogicScriptStatement::IfStatement(
                if_statement,
            )))
            .chain(subsequent_code.into_iter())
            .collect::<Vec<_>>())
    }

    fn generate_logic_script_for_basic_block(
        &mut self,
        block_id: NodeIndex,
        queue: &mut VecDeque<NodeIndex>,
    ) -> Result<Vec<LogicScriptStatement<ParsedLogicArgument>>, LogicScriptCodeGenerationError>
    {
        if self.visited_basic_blocks.contains(&block_id) {
            let label = self
                .context
                .block_labels
                .get(&block_id)
                .cloned()
                .ok_or_else(|| {
                    LogicScriptCodeGenerationError::JumpToUnlabeledStatement(
                        block_id,
                        self.context.get_block(block_id).cloned(),
                    )
                })?;

            return Ok(vec![self.generate_goto(label)]);
        }

        self.visited_basic_blocks.insert(block_id);

        let control_flow = self
            .context
            .basic_block_graph
            .control_flow_for_block(block_id)?;

        match control_flow {
            BasicBlockControlFlow::SinglePath { next_id, .. } => {
                self.generate_single_path_code(block_id, next_id, queue)
            }
            BasicBlockControlFlow::Conditional {
                conditions,
                then_id,
                else_id,
                ..
            } => self.generate_conditional_code(block_id, &conditions, then_id, else_id, queue),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::{fs::File, io::Write};

    use crate::{
        agi_version::AGIVersion,
        logic::{logic_script::codegen::context::LogicScriptCodeGenerationContext, LogicProgram},
        resources::{decode::Decode, file_provider::FileProvider, ResourceType},
        test_data::{uriquest_dir, uriquest_resources},
        word_list::WordList,
    };

    #[test]
    fn smoke_test() {
        let resources = uriquest_resources();
        let logic_data = resources
            .read_resource_data(ResourceType::LOGIC, 0)
            .expect("Failed to read logic data");
        let logic = LogicProgram::decode_from_bytes(&logic_data, &AGIVersion::new(2, 917))
            .expect("Failed to decode logic program");
        let word_list = WordList::decode_from_bytes(
            &uriquest_dir()
                .read_file_bytes("WORDS.TOK")
                .expect("Failed to read WORDS.TOK"),
            (),
        )
        .expect("Failed to decode word list");

        let context = LogicScriptCodeGenerationContext::try_from_program(&logic, &word_list)
            .expect("Failed to create code generation context");

        eprintln!("Generated context!");

        File::create("debug.dot")
            .expect("Failed to open debug.dot for writing")
            .write_fmt(format_args!(
                "{}",
                context.basic_block_graph.to_dot(&context.asm_context)
            ))
            .expect("Failed to write dot diagram");

        // let generator = LogicScriptProgramGenerator::new(&context);

        // let statements = generator
        //     .generate_statements()
        //     .expect("Failed to generate logic script statements");

        // panic!(
        //     "{}",
        //     statements
        //         .into_iter()
        //         .map(|s| s.generate_logic_script(&context, 0))
        //         .collect::<Result<String, _>>()
        //         .expect("Error generating logic script")
        // );
    }
}

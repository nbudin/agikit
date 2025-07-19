use std::collections::{HashSet, VecDeque};

use petgraph::graph::NodeIndex;

use crate::logic::{
    LogicConditionClause, LogicMessages,
    analysis::{
        basic_block_graph::{BasicBlock, BasicBlockControlFlow},
        optimization::Optimizable,
    },
    asm::{
        codegen::generate_logic_messages,
        expressions::{LogicBooleanExpression, LogicIdentifier, ParsedLogicArgument},
    },
    logic_script::{
        codegen::{
            codegen::GenerateLogicScript, command_to_statement::CommandToStatement,
            context::LogicScriptCodeGenerationContext, errors::LogicScriptCodeGenerationError,
            statement_graph::LogicScriptStatementGraph,
        },
        identifiers::IdentifierMap,
        statements::{
            KeywordType, LogicScriptCommandCall, LogicScriptIfStatement, LogicScriptKeyword,
            LogicScriptLabel, LogicScriptStatement,
        },
    },
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

    pub fn generate_logic_script(
        self,
        messages: &LogicMessages,
    ) -> Result<String, LogicScriptCodeGenerationError> {
        let context = self.context;
        let statements = self.generate_statements()?;
        let mut statement_graph =
            LogicScriptStatementGraph::try_from_statements(&statements, IdentifierMap::builtins())?;
        statement_graph.optimize();

        let optimized_statements = statement_graph.to_statements()?;
        #[cfg(test)]
        crate::test_utils::write_and_edit(
            ".dot",
            &statement_graph.to_dot(context),
            // &LogicScriptStatementGraph::try_from_statements(
            //     &optimized_statements,
            //     IdentifierMap::builtins(),
            // )?
            // .to_dot(context),
        );
        let program_section = optimized_statements.generate_logic_script(context, 0)?;

        Ok(format!(
            "{}\n\n\n{}\n",
            program_section.trim(),
            generate_logic_messages(messages)?
        ))
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

        Ok(statements)
    }

    fn generate_goto(&self, label: String) -> LogicScriptStatement<ParsedLogicArgument> {
        LogicScriptStatement::CommandCall(LogicScriptCommandCall {
            command_name: "goto".to_string(),
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
            if self
                .context
                .domination_analysis
                .dominates(block_id, next_block_id)
                && self
                    .context
                    .domination_analysis
                    .post_dominates(next_block_id, block_id)
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
                if block_id == to_block_id
                    || !self
                        .context
                        .domination_analysis
                        .dominates(block_id, to_block_id)
                {
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
                    && !self
                        .context
                        .domination_analysis
                        .post_dominates(branch_queue[0], to_block_id)
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
            if self
                .context
                .domination_analysis
                .dominates(block_id, else_id)
                && then_queue.iter().all(|next_block_id| {
                    self.context
                        .domination_analysis
                        .dominates(block_id, *next_block_id)
                })
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

            if !self
                .context
                .domination_analysis
                .dominates(block_id, inner_block_id)
            {
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
            let label = self.context.label_for_block_id(block_id);
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
    use crate::{
        logic::logic_script::codegen::{
            context::LogicScriptCodeGenerationContext,
            program_generator::LogicScriptProgramGenerator,
        },
        resources::file_provider::FileProvider,
        test_data::uriquest,
    };

    use similar_asserts::assert_eq;

    macro_rules! logic_smoke_test {
        ($test_name: ident, $resource_number: literal) => {
            #[test]
            fn $test_name() {
                let uriquest = uriquest();
                let logic = uriquest
                    .decode_logic($resource_number)
                    .expect("Failed to decode logic program");
                let word_list = uriquest
                    .decode_word_list()
                    .expect("Failed to decode word list");

                let context =
                    LogicScriptCodeGenerationContext::try_from_program(&logic, &word_list)
                        .expect("Failed to create code generation context");

                let generator = LogicScriptProgramGenerator::new(&context);

                let generated_script = generator
                    .generate_logic_script(&logic.messages)
                    .expect("Failed to generate script");

                let expected = uriquest
                    .read_file_utf8(&format!("src/logic/{}.agilogic", $resource_number))
                    .unwrap();

                assert_eq!(expected, generated_script);
            }
        };
    }

    logic_smoke_test!(logic_0_test, 0);
    logic_smoke_test!(logic_13_test, 13);
    logic_smoke_test!(logic_93_test, 93);
    logic_smoke_test!(logic_99_test, 99);
    logic_smoke_test!(logic_100_test, 100);
}

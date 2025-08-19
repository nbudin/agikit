use std::collections::{HashMap, hash_map::Entry};

use petgraph::{
    Direction,
    graph::NodeIndex,
    visit::{Dfs, Walker},
};

use crate::{
    agi_version::AGIVersion,
    logic::{
        LogicCommand, LogicCondition, LogicGoto, LogicInstruction, LogicProgram,
        analysis::{
            ast::{LogicASTNodeMetadata, LogicCommandNode},
            basic_block_graph::{
                BasicBlock, BasicBlockEdgeType, BasicBlockGraph, SinglePathBasicBlock,
            },
            dominator_tree::{DominationAnalysis, ImmediatePostDominator},
            optimization::{DirectedNeighborEdgeUtils, Optimizable},
        },
        asm::LogicLabel,
        commands::AGICommand,
        logic_script::{
            codegen::{node_label_map::NodeLabelMap, statement_graph::LogicScriptStatementGraph},
            compile::{
                ast_generator::LogicScriptASTGenerator,
                diagnostics::LogicScriptDiagnostic,
                errors::CompilationError,
                post_compilation_passes::{
                    PostCompilationPass, RemoveRedundantGotoInstructionsPass,
                    make_conditionals_self_contained, remove_unreachable_instructions,
                },
                preprocess::{parse_logic_script_raw, preprocess_logic_script},
                primitive_statements::LogicScriptPrimitiveStatement,
            },
        },
    },
    object_list::ObjectList,
    resources::file_provider::FileProvider,
    word_list::WordList,
};

#[derive(Debug, Clone)]
enum CompiledBlock {
    SinglePath {
        #[allow(unused)]
        basic_block_id: NodeIndex,
        instructions: Vec<LogicInstruction>,
        next_id: Option<NodeIndex>,
    },
    Conditional {
        #[allow(unused)]
        basic_block_id: NodeIndex,
        condition: LogicCondition,
        skip_id: NodeIndex,
        then_id: Option<NodeIndex>,
    },
}

pub struct LogicCompiler {
    basic_block_graph: BasicBlockGraph,
    label_map: NodeLabelMap,
    labels: HashMap<u16, LogicLabel>,
    agi_version: AGIVersion,
    domination_analysis: DominationAnalysis,
    compiled_blocks: HashMap<NodeIndex, CompiledBlock>,
    stitched_blocks: HashMap<NodeIndex, Vec<LogicInstruction>>,
    instructions_by_address: HashMap<u16, LogicInstruction>,
}

impl LogicCompiler {
    pub fn new(
        basic_block_graph: BasicBlockGraph,
        label_map: NodeLabelMap,
        agi_version: &AGIVersion,
    ) -> Self {
        let domination_analysis =
            DominationAnalysis::from_graph(&basic_block_graph.graph, basic_block_graph.root_id());
        Self {
            basic_block_graph,
            label_map,
            labels: HashMap::new(),
            agi_version: agi_version.clone(),
            domination_analysis,
            compiled_blocks: HashMap::new(),
            stitched_blocks: HashMap::new(),
            instructions_by_address: HashMap::new(),
        }
    }

    pub fn compile(mut self) -> Result<(Vec<LogicInstruction>, Vec<LogicLabel>), CompilationError> {
        let basic_block_ids = Dfs::new(
            &self.basic_block_graph.graph,
            self.basic_block_graph.root_block_id,
        )
        .iter(&self.basic_block_graph.graph)
        .collect::<Vec<_>>();

        for block_id in basic_block_ids.into_iter() {
            self.compile_block(block_id)?;
        }

        let mut instructions = self.stitch_blocks(self.basic_block_graph.root_block_id)?;
        for mut pass in self.post_compilation_passes() {
            pass.run_until_done(&mut instructions);
        }

        Ok((instructions, self.labels.values().cloned().collect()))
    }

    pub fn post_compilation_passes(&self) -> Vec<Box<dyn PostCompilationPass>> {
        vec![
            Box::new(remove_unreachable_instructions),
            Box::new(RemoveRedundantGotoInstructionsPass::new()),
            // TODO add an option to disable this pass
            Box::new(make_conditionals_self_contained),
        ]
    }

    fn store_instruction(
        &mut self,
        instruction: LogicInstruction,
    ) -> Result<LogicInstruction, CompilationError> {
        let entry = self.instructions_by_address.entry(instruction.address());
        match entry {
            Entry::Occupied(_) => Err(CompilationError::ConflictingInstructionForAddress(
                instruction.address(),
            )),
            Entry::Vacant(vacant_entry) => {
                vacant_entry.insert(instruction.clone());
                Ok(instruction)
            }
        }
    }

    fn find_free_address(&self) -> u16 {
        self.instructions_by_address
            .keys()
            .max()
            .map(|max_address| max_address + 1)
            .unwrap_or(0)
    }

    fn find_block_address(&self, block_id: NodeIndex) -> Result<u16, CompilationError> {
        let Some(block) = self.compiled_blocks.get(&block_id) else {
            return Err(CompilationError::BlockHasNotBeenCompiled(block_id));
        };

        match block {
            CompiledBlock::SinglePath {
                instructions,
                next_id,
                ..
            } => {
                if let Some(first_instruction) = instructions.first() {
                    return Ok(first_instruction.address());
                }

                if let Some(stitched_block) = self.stitched_blocks.get(&block_id)
                    && let Some(first_instruction) = stitched_block.first()
                {
                    return Ok(first_instruction.address());
                }
                if let Some(next_id) = next_id {
                    return self.find_block_address(*next_id);
                } else {
                    return Err(CompilationError::CannotFindAddressForEmptyBlock(block_id));
                }
            }
            CompiledBlock::Conditional { condition, .. } => Ok(condition.address),
        }
    }

    fn find_or_build_label_for_address(&mut self, address: u16) -> LogicLabel {
        let entry = self.labels.entry(address);
        match entry {
            Entry::Occupied(entry) => entry.get().clone(),
            Entry::Vacant(entry) => entry
                .insert(LogicLabel {
                    address,
                    label: format!("GeneratedLabel{address}"),
                })
                .clone(),
        }
    }

    fn stitch_blocks(
        &mut self,
        block_id: NodeIndex,
    ) -> Result<Vec<LogicInstruction>, CompilationError> {
        let block = self.compiled_blocks.get(&block_id).unwrap().clone();

        if let Some(_) = self.stitched_blocks.get(&block_id) {
            let block_address = self.find_block_address(block_id)?;
            self.find_or_build_label_for_address(block_address);
            let goto_instruction = LogicInstruction::Goto(LogicGoto {
                address: self.find_free_address(),
                jump_address: block_address,
            });
            let goto_instruction = self.store_instruction(goto_instruction)?;
            return Ok(vec![goto_instruction]);
        }

        let instructions = match block {
            CompiledBlock::SinglePath {
                instructions,
                next_id,
                ..
            } => {
                let mut instructions = instructions.clone();
                if let Some(next_id) = next_id {
                    instructions.extend(self.stitch_blocks(next_id)?);
                }
                instructions
            }
            CompiledBlock::Conditional {
                condition,
                skip_id,
                then_id,
                ..
            } => {
                let skip_instructions = self.stitch_blocks(skip_id)?;
                let skip_address = self.find_block_address(skip_id)?;
                self.find_or_build_label_for_address(skip_address);

                let if_instruction = LogicInstruction::Condition(condition.clone());
                let if_instruction = self.store_instruction(if_instruction)?;

                let mut instructions = vec![if_instruction];
                if let Some(then_id) = then_id {
                    instructions.extend(self.stitch_blocks(then_id)?);
                }
                instructions.extend(skip_instructions);

                instructions
            }
        };

        self.stitched_blocks.insert(block_id, instructions.clone());
        Ok(instructions)
    }

    fn compile_command_node(
        &mut self,
        command_node: LogicCommandNode,
    ) -> Result<LogicInstruction, CompilationError> {
        let instruction = LogicInstruction::Command(LogicCommand {
            agi_command: command_node.command.agi_command,
            address: command_node.command.address,
            args: command_node.command.args,
        });

        self.store_instruction(instruction)
    }

    fn generate_else_virtual_return(
        &mut self,
        block_id: NodeIndex,
    ) -> Result<NodeIndex, CompilationError> {
        let command = LogicCommand {
            address: self.find_free_address(),
            agi_command: AGICommand::by_name("return", &self.agi_version)
                .unwrap()
                .clone(),
            args: vec![],
        };
        self.store_instruction(LogicInstruction::Command(command.clone()))?;
        let node = LogicCommandNode {
            metadata: LogicASTNodeMetadata {
                instruction_address: Some(command.address),
            },
            command: command,
            label: None,
        };
        let block = SinglePathBasicBlock {
            label: None,
            commands: vec![node],
        };
        let return_block_id = self
            .basic_block_graph
            .graph
            .add_node(BasicBlock::SinglePath(block));
        self.basic_block_graph.graph.add_edge(
            block_id,
            return_block_id,
            BasicBlockEdgeType::IfElse,
        );
        Ok(return_block_id)
    }

    fn compile_block(&mut self, block_id: NodeIndex) -> Result<(), CompilationError> {
        if let Some(_) = self.compiled_blocks.get(&block_id) {
            return Ok(());
        }

        let block = self
            .basic_block_graph
            .graph
            .node_weight(block_id)
            .unwrap()
            .clone();

        let compiled_block = match block {
            BasicBlock::SinglePath(block) => {
                let instructions = block
                    .commands
                    .iter()
                    .cloned()
                    .map(|command_node| self.compile_command_node(command_node))
                    .collect::<Result<Vec<_>, _>>()?;

                CompiledBlock::SinglePath {
                    basic_block_id: block_id,
                    instructions,
                    next_id: self
                        .basic_block_graph
                        .graph
                        .directed_neighbor_node_id_of_type(
                            block_id,
                            Direction::Outgoing,
                            BasicBlockEdgeType::Next,
                        ),
                }
            }
            BasicBlock::Conditional(block) => {
                let Some(next_block_id) =
                    self.domination_analysis.immediate_post_dominator(block_id)
                else {
                    return Err(CompilationError::CannotFindNextBlockAfterIf(block_id));
                };

                let skip_id = if let Some(else_id) = self
                    .basic_block_graph
                    .graph
                    .directed_neighbor_node_id_of_type(
                        block_id,
                        Direction::Outgoing,
                        BasicBlockEdgeType::IfElse,
                    ) {
                    else_id
                } else {
                    match next_block_id {
                        ImmediatePostDominator::Node(node_index) => node_index,
                        ImmediatePostDominator::VirtualRoot => {
                            self.generate_else_virtual_return(block_id)?
                        }
                    }
                };

                CompiledBlock::Conditional {
                    basic_block_id: block_id,
                    condition: LogicCondition {
                        address: self.find_free_address(),
                        clauses: block.conditions,
                        skip_address: 0,
                    },
                    skip_id,
                    then_id: self
                        .basic_block_graph
                        .graph
                        .directed_neighbor_node_id_of_type(
                            block_id,
                            Direction::Outgoing,
                            BasicBlockEdgeType::IfThen,
                        ),
                }
            }
        };

        self.compiled_blocks.insert(block_id, compiled_block);
        Ok(())
    }
}

pub fn compile_logic_script<FP: FileProvider>(
    source_code: &str,
    script_path: &str,
    word_list: &WordList,
    object_list: &ObjectList,
    agi_version: &AGIVersion,
    file_provider: &FP,
) -> Result<(LogicProgram, Vec<LogicScriptDiagnostic>), CompilationError> {
    let raw_program = parse_logic_script_raw(source_code, agi_version)?;
    let (preprocessed_program, identifier_map) = preprocess_logic_script(
        raw_program.as_slice(),
        script_path,
        agi_version,
        file_provider,
    )?;

    let mut statement_graph = LogicScriptStatementGraph::try_from_statements(
        preprocessed_program.as_slice(),
        identifier_map,
    )?;
    statement_graph.optimize();
    let (primitive_statements, identifiers) =
        LogicScriptPrimitiveStatement::simplify_statement_graph(statement_graph)?;
    let primitive_statement_graph = LogicScriptStatementGraph::try_from_statements(
        primitive_statements.as_slice(),
        identifiers.clone(),
    )?;

    let ast_generator = LogicScriptASTGenerator::new(
        primitive_statement_graph,
        identifiers,
        word_list.clone(),
        object_list.clone(),
        agi_version.clone(),
    );
    let (ast, label_map, messages) = ast_generator.generate()?;
    let basic_block_graph = BasicBlockGraph::from_ast(&ast);
    let (instructions, _labels) =
        LogicCompiler::new(basic_block_graph, label_map, agi_version).compile()?;

    Ok((
        LogicProgram {
            instructions,
            messages,
        },
        vec![],
    ))
}

#[cfg(test)]
mod tests {
    use crate::{
        logic::logic_script::compile::compile::compile_logic_script,
        resources::file_provider::FileProvider, test_data::uriquest,
    };

    #[test]
    fn smoke_test() {
        let uriquest = uriquest();
        compile_logic_script(
            uriquest
                .read_file_utf8("src/logic/0.agilogic")
                .unwrap()
                .as_str(),
            "src/logic/0.agilogic",
            &uriquest.decode_word_list().unwrap(),
            &uriquest.decode_object_list().unwrap(),
            &uriquest.config.agi_version,
            &uriquest,
        )
        .unwrap();
    }
}

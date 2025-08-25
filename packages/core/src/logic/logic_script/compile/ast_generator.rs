use std::collections::HashMap;

use petgraph::{
    graph::NodeIndex,
    prelude::StableDiGraph,
    visit::{Control, Dfs, DfsEvent},
};

use crate::{
    agi_version::AGIVersion,
    logic::{
        LogicCommand, LogicConditionClause, LogicMessages, LogicOr, LogicTest,
        analysis::ast::{
            LogicAST, LogicASTEdge, LogicASTNode, LogicASTNodeMetadata, LogicCommandNode,
            LogicGotoNode, LogicIfNode,
        },
        asm::{
            LogicLabel,
            expressions::{LogicBooleanExpression, LogicTestCall, ParsedLogicArgument},
            literals::LogicLiteralValue,
        },
        commands::{AGICommand, AGICommandArgType, TestCommand},
        logic_script::{
            codegen::{
                node_label_map::NodeLabelMap,
                statement_graph::{
                    LogicScriptStatementGraph, LogicScriptStatementGraphEdge,
                    LogicScriptStatementGraphNode,
                },
            },
            compile::{
                primitive_expressions::{PrimitiveBooleanExpression, PrimitiveOrClause},
                primitive_statements::{
                    LogicScriptPrimitiveStatement, LogicScriptPrimitiveStatementBody,
                },
            },
            directives::Directive,
            identifiers::{IdentifierMap, IdentifierMapping},
        },
    },
    object_list::ObjectList,
    word_list::WordList,
};

#[derive(Debug, Clone)]
pub enum ASTGenerationError {
    BooleanOperationCannotHaveTwoLiteralOperands,
    InvalidValueForArgType {
        value: LogicLiteralValue,
        arg_type: AGICommandArgType,
    },
    StatementAddressNotFound(NodeIndex),
    StatementNodeNotFound(NodeIndex),
    StringLiteralInBooleanOperation,
    TypeMismatch {
        expected: AGICommandArgType,
        got: AGICommandArgType,
    },
    UnknownCommand(String),
    UnknownIdentifier(String),
    UnknownLabel(String),
    UnknownObjectName(String),
    UnknownTestCommand(String),
    UnknownWord(String),
}

struct UnresolvedGoto {
    ast_node_id: NodeIndex,
    target_label: String,
}

pub struct LogicScriptASTGenerator {
    statement_graph: LogicScriptStatementGraph<LogicScriptPrimitiveStatement>,
    ast_graph: StableDiGraph<LogicASTNode, LogicASTEdge>,
    ast_node_id_by_statement_node_id: HashMap<NodeIndex, NodeIndex>,
    identifiers: IdentifierMap,
    inverted_word_list: HashMap<String, u16>,
    object_numbers_by_name: HashMap<String, usize>,
    label_map: NodeLabelMap,
    unresolved_gotos: Vec<UnresolvedGoto>,
    address_by_node_id: HashMap<NodeIndex, usize>,
    node_id_by_address: HashMap<usize, NodeIndex>,
    messages_by_number: HashMap<usize, String>,
    message_numbers_by_content: HashMap<String, usize>,
    agi_version: AGIVersion,
}

impl LogicScriptASTGenerator {
    pub fn new(
        statement_graph: LogicScriptStatementGraph<LogicScriptPrimitiveStatement>,
        identifiers: IdentifierMap,
        directives: Vec<Directive>,
        word_list: WordList,
        object_list: ObjectList,
        agi_version: AGIVersion,
    ) -> Self {
        let inverted_word_list = word_list
            .words
            .iter()
            .flat_map(|(word_number, entry)| {
                entry
                    .iter_words()
                    .map(|word| (word.to_string(), *word_number))
            })
            .collect::<HashMap<_, _>>();
        let object_numbers_by_name = object_list
            .objects
            .iter()
            .enumerate()
            .map(|(object_number, entry)| (entry.name.clone(), object_number))
            .collect::<HashMap<_, _>>();

        let mut generator = Self {
            identifiers,
            label_map: NodeLabelMap::new(&statement_graph.graph, statement_graph.root_id),
            statement_graph,
            ast_graph: StableDiGraph::new(),
            ast_node_id_by_statement_node_id: HashMap::new(),
            inverted_word_list,
            object_numbers_by_name,
            unresolved_gotos: vec![],
            node_id_by_address: HashMap::new(),
            address_by_node_id: HashMap::new(),
            messages_by_number: HashMap::new(),
            message_numbers_by_content: HashMap::new(),
            agi_version,
        };

        let mut dfs = Dfs::new(
            &generator.statement_graph.graph,
            generator.statement_graph.root_id,
        );
        let mut address = 1;
        while let Some(node_id) = dfs.next(&generator.statement_graph.graph) {
            generator.add_node_address(node_id, address);
            address += 10;
        }
        for directive in directives {
            match directive {
                Directive::Message { number, message } => {
                    generator.add_message(number.value as usize, message.value())
                }
                _ => {}
            }
        }

        generator
    }

    fn add_node_address(&mut self, node_id: NodeIndex, address: usize) {
        self.node_id_by_address.insert(address, node_id);
        self.address_by_node_id.insert(node_id, address);
    }

    fn add_message(&mut self, message_number: usize, message: String) {
        self.message_numbers_by_content
            .insert(message.clone(), message_number);
        self.messages_by_number.insert(message_number, message);
    }

    fn get_message_number(&mut self, message: String) -> usize {
        if let Some(message_number) = self.message_numbers_by_content.get(&message) {
            return *message_number;
        }

        let new_message_number = self.message_numbers_by_content.values().max().unwrap_or(&0) + 1;
        self.add_message(new_message_number, message);
        new_message_number
    }

    fn encode_argument(
        &mut self,
        argument: &ParsedLogicArgument,
        argument_type: AGICommandArgType,
    ) -> Result<u16, ASTGenerationError> {
        Ok(match argument {
            ParsedLogicArgument::Literal(literal) => match &literal.value {
                LogicLiteralValue::Number(number) => number.value as u16,
                LogicLiteralValue::String(string) => match argument_type {
                    AGICommandArgType::Message => {
                        self.get_message_number(string.value.clone()) as u16
                    }
                    AGICommandArgType::Item => self
                        .object_numbers_by_name
                        .get(&string.value)
                        .map(|object_number| *object_number as u16)
                        .ok_or_else(|| {
                            ASTGenerationError::UnknownObjectName(string.value.clone())
                        })?,
                    AGICommandArgType::Word => self
                        .inverted_word_list
                        .get(&string.value)
                        .map(|word_number| *word_number as u16)
                        .ok_or_else(|| ASTGenerationError::UnknownWord(string.value.clone()))?,
                    _ => {
                        return Err(ASTGenerationError::InvalidValueForArgType {
                            value: literal.value.clone(),
                            arg_type: argument_type,
                        });
                    }
                },
            },
            ParsedLogicArgument::Identifier(identifier) => {
                let Some(mapping) = self.identifiers.get(&identifier.name) else {
                    return Err(ASTGenerationError::UnknownIdentifier(
                        identifier.name.clone(),
                    ));
                };

                match mapping {
                    IdentifierMapping::Variable {
                        name: _,
                        number,
                        variable_type,
                    } => {
                        if *variable_type != argument_type {
                            return Err(ASTGenerationError::TypeMismatch {
                                expected: argument_type,
                                got: *variable_type,
                            });
                        }

                        *number
                    }
                    IdentifierMapping::ConstantString { value, .. } => {
                        self.get_message_number(value.clone()) as u16
                    }
                    IdentifierMapping::ConstantNumber { value, .. } => *value as u16,
                }
            }
        })
    }

    fn test_call_to_test(
        &mut self,
        expression: &LogicTestCall<ParsedLogicArgument>,
    ) -> Result<LogicTest, ASTGenerationError> {
        let Some(test_command) = TestCommand::by_name(&expression.test_name) else {
            return Err(ASTGenerationError::UnknownTestCommand(
                expression.test_name.clone(),
            ));
        };

        let args = expression
            .argument_list
            .iter()
            .enumerate()
            .map(|(index, arg)| {
                self.encode_argument(
                    arg,
                    if test_command.name == "said" {
                        AGICommandArgType::Word
                    } else {
                        test_command.arg_types[index]
                    },
                )
            })
            .collect::<Result<_, _>>()?;

        Ok(LogicTest {
            test_command: test_command.clone(),
            args,
            negate: false,
        })
    }

    fn primitive_boolean_expression_to_clauses(
        &mut self,
        primitive_expression: &PrimitiveBooleanExpression,
    ) -> Result<Vec<LogicConditionClause>, ASTGenerationError> {
        Ok(match primitive_expression {
            PrimitiveBooleanExpression::TestCall(test_call) => {
                vec![LogicConditionClause::Test(
                    self.test_call_to_test(&test_call)?,
                )]
            }
            PrimitiveBooleanExpression::NotTestCall(test_call) => {
                let test = self.test_call_to_test(&test_call)?;
                vec![LogicConditionClause::Test(LogicTest {
                    negate: true,
                    ..test
                })]
            }
            PrimitiveBooleanExpression::Or(or_clauses) => {
                let tests = or_clauses
                    .into_iter()
                    .map(|or_clause| match or_clause {
                        PrimitiveOrClause::TestCall(test_call) => {
                            Ok(vec![self.test_call_to_test(&test_call)?])
                        }
                        PrimitiveOrClause::NotTestCall(test_call) => {
                            let clause = self.test_call_to_test(&test_call)?;
                            Ok(vec![LogicTest {
                                negate: true,
                                ..clause
                            }])
                        }
                    })
                    .collect::<Result<Vec<_>, _>>()?
                    .into_iter()
                    .flatten()
                    .collect();
                vec![LogicConditionClause::Or(LogicOr { or_tests: tests })]
            }
            PrimitiveBooleanExpression::And(and_clauses) => and_clauses
                .into_iter()
                .map(|and_clause| {
                    self.primitive_boolean_expression_to_clauses(&and_clause.clone().into())
                })
                .collect::<Result<Vec<_>, _>>()?
                .into_iter()
                .flatten()
                .collect(),
        })
    }

    fn boolean_expression_to_clauses(
        &mut self,
        conditions: &LogicBooleanExpression<ParsedLogicArgument>,
    ) -> Result<Vec<LogicConditionClause>, ASTGenerationError> {
        let primitive_expression = PrimitiveBooleanExpression::try_from_logic_boolean_expression(
            conditions,
            &self.identifiers,
        )?;

        self.primitive_boolean_expression_to_clauses(&primitive_expression)
    }

    fn add_ast_node(&mut self, node: LogicASTNode, statement_node_id: NodeIndex) -> NodeIndex {
        let node_id = self.ast_graph.add_node(node);
        self.ast_node_id_by_statement_node_id
            .insert(statement_node_id, node_id);
        node_id
    }

    fn generate_ast_for_statement(
        &mut self,
        statement_node_id: NodeIndex,
        agi_version: &AGIVersion,
    ) -> Result<Option<NodeIndex>, ASTGenerationError> {
        if let Some(existing_node_id) = self
            .ast_node_id_by_statement_node_id
            .get(&statement_node_id)
        {
            return Ok(Some(*existing_node_id));
        }

        let Some(address) = self.address_by_node_id.get(&statement_node_id) else {
            return Err(ASTGenerationError::StatementAddressNotFound(
                statement_node_id,
            ));
        };

        let Some(statement) = self.statement_graph.graph.node_weight(statement_node_id) else {
            return Err(ASTGenerationError::StatementNodeNotFound(statement_node_id));
        };
        let statement = statement.clone();

        let label = statement.label.as_ref().map(|l| LogicLabel {
            address: *address as u16,
            label: l.clone(),
        });
        let metadata = LogicASTNodeMetadata {
            instruction_address: Some(*address as u16),
        };

        if let Some(goto_target_label) = statement.get_goto_target_label() {
            let node = LogicASTNode::Goto(LogicGotoNode { label, metadata });
            let ast_node_id = self.add_ast_node(node, statement_node_id);

            self.unresolved_gotos.push(UnresolvedGoto {
                ast_node_id,
                target_label: goto_target_label.to_string(),
            });

            return Ok(Some(ast_node_id));
        }

        let node = match &statement.body {
            LogicScriptPrimitiveStatementBody::CommandCall(body) => {
                let Some(agi_command) = AGICommand::by_name(&body.command_name, agi_version) else {
                    return Err(ASTGenerationError::UnknownCommand(
                        body.command_name.clone(),
                    ));
                };

                Some(LogicASTNode::Command(LogicCommandNode {
                    command: LogicCommand {
                        address: *address as u16,
                        agi_command: agi_command.clone(),
                        args: body
                            .argument_list
                            .iter()
                            .enumerate()
                            .map(|(index, arg)| {
                                self.encode_argument(arg, agi_command.arg_types[index])
                                    .map(|encoded| encoded as u8)
                            })
                            .collect::<Result<_, _>>()?,
                    },
                    label,
                    metadata,
                }))
            }
            LogicScriptPrimitiveStatementBody::IfStatement(body) => {
                Some(LogicASTNode::If(LogicIfNode {
                    clauses: self.boolean_expression_to_clauses(&body.conditions)?,
                    label,
                    metadata,
                }))
            }
        };

        Ok(node.map(|node| self.add_ast_node(node, statement_node_id)))
    }

    pub fn generate(
        mut self,
    ) -> Result<(LogicAST, NodeLabelMap, LogicMessages), ASTGenerationError> {
        let agi_version = self.agi_version.clone();
        let graph = self.statement_graph.graph.clone();

        // pass 1: generate AST nodes
        let mut dfs = Dfs::new(&graph, self.statement_graph.root_id);
        while let Some(statement_node_id) = dfs.next(&graph) {
            self.generate_ast_for_statement(statement_node_id, &agi_version)?;
        }

        // pass 2: connect nodes
        let result =
            petgraph::visit::depth_first_search(&graph, [self.statement_graph.root_id], |event| {
                let mut handle_event = || match event {
                    DfsEvent::TreeEdge(from_statement_id, to_statement_id) => {
                        let from_ast_node_id = self
                            .ast_node_id_by_statement_node_id
                            .get(&from_statement_id)
                            .copied()
                            .unwrap();
                        let to_ast_node_id = self
                            .ast_node_id_by_statement_node_id
                            .get(&to_statement_id)
                            .copied()
                            .unwrap();

                        for edge in self
                            .statement_graph
                            .graph
                            .edges_connecting(from_statement_id, to_statement_id)
                        {
                            match edge.weight() {
                                LogicScriptStatementGraphEdge::Next => {
                                    self.ast_graph.add_edge(
                                        from_ast_node_id,
                                        to_ast_node_id,
                                        LogicASTEdge::CommandToNext,
                                    );
                                }
                                LogicScriptStatementGraphEdge::GotoTarget => {
                                    // This will be handled in the resolve gotos pass after this
                                }
                                LogicScriptStatementGraphEdge::IfThen => {
                                    self.ast_graph.add_edge(
                                        from_ast_node_id,
                                        to_ast_node_id,
                                        LogicASTEdge::IfThen,
                                    );
                                }
                                LogicScriptStatementGraphEdge::IfElse => {
                                    self.ast_graph.add_edge(
                                        from_ast_node_id,
                                        to_ast_node_id,
                                        LogicASTEdge::IfElse,
                                    );
                                }
                                LogicScriptStatementGraphEdge::BlockExit => {
                                    self.ast_graph.add_edge(
                                        from_ast_node_id,
                                        to_ast_node_id,
                                        LogicASTEdge::CommandToNext,
                                    );
                                }
                            }
                        }

                        Ok(())
                    }
                    _ => Ok(()),
                };

                match handle_event() {
                    Ok(_) => Control::Continue,
                    Err(error) => Control::Break(error),
                }
            });

        if let Control::Break(error) = result {
            return Err(error);
        }

        // pass 3: resolve gotos
        for unresolved_goto in self.unresolved_gotos {
            let Some(target_statement_id) = self
                .label_map
                .get_node_id_for_label(&unresolved_goto.target_label)
            else {
                return Err(ASTGenerationError::UnknownLabel(
                    unresolved_goto.target_label.clone(),
                ));
            };
            let target_node_id = *self
                .ast_node_id_by_statement_node_id
                .get(&target_statement_id)
                .unwrap();
            self.ast_graph.add_edge(
                unresolved_goto.ast_node_id,
                target_node_id,
                LogicASTEdge::GotoToTarget,
            );
        }

        Ok((
            LogicAST {
                graph: self.ast_graph,
                root_node_id: *self
                    .ast_node_id_by_statement_node_id
                    .get(&self.statement_graph.root_id)
                    .unwrap(),
                nodes_by_address: self
                    .node_id_by_address
                    .into_iter()
                    .map(|(address, node_id)| (address as u16, node_id))
                    .collect(),
            },
            self.label_map,
            self.messages_by_number
                .into_iter()
                .map(|(number, message)| (number as u8, message))
                .collect(),
        ))
    }
}

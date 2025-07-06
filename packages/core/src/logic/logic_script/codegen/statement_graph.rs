use std::collections::{HashMap, HashSet, VecDeque};
#[cfg(feature = "dot")]
use std::fmt::Debug;

use petgraph::{
    Direction,
    algo::has_path_connecting,
    graph::{DiGraph, NodeIndex},
    visit::{Dfs, DfsPostOrder, EdgeFiltered, EdgeRef, Walker},
};

#[cfg(feature = "dot")]
use crate::logic::logic_script::codegen::context::LogicScriptCodeGenerationContext;
use crate::logic::{
    analysis::optimization::{
        DirectedNeighborEdgeUtils, Optimizable, OptimizationVisitor, RemoveNodePreservingEdges,
    },
    asm::expressions::{
        LogicBooleanExpression, LogicIdentifier, LogicNotExpression, ParsedLogicArgument,
    },
    logic_script::{
        codegen::errors::LogicScriptCodeGenerationError,
        identifiers::IdentifierMap,
        statements::{LogicScriptCommandCall, LogicScriptIfStatement, LogicScriptStatement},
    },
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogicScriptStatementGraphEdge {
    Next,
    GotoTarget,
    IfThen,
    IfElse,
    BlockExit,
}

fn add_statements_to_graph<'a>(
    graph: &mut DiGraph<LogicScriptStatement<ParsedLogicArgument>, LogicScriptStatementGraphEdge>,
    statements: Box<dyn Iterator<Item = &'a LogicScriptStatement<ParsedLogicArgument>> + 'a>,
) -> (Vec<NodeIndex>, Vec<NodeIndex>) {
    let mut prev_node_id: Option<NodeIndex> = None;
    let mut block_exit_node_ids: Vec<NodeIndex> = vec![];
    let added_statements: Vec<_> = statements
        .map(|statement| {
            let node_id = graph.add_node(statement.clone());

            for block_exit_node_id in block_exit_node_ids.iter() {
                graph.add_edge(
                    *block_exit_node_id,
                    node_id,
                    LogicScriptStatementGraphEdge::BlockExit,
                );
            }
            block_exit_node_ids.clear();

            if let Some(prev_node_id) = prev_node_id {
                graph.add_edge(prev_node_id, node_id, LogicScriptStatementGraphEdge::Next);
            }
            prev_node_id = Some(node_id);

            match statement {
                LogicScriptStatement::IfStatement(if_statement) => {
                    let then_statements = if_statement
                        .then_statements
                        .iter()
                        .map(|stmt| stmt.as_ref());
                    let (then_statement_ids, then_block_exits) =
                        add_statements_to_graph(graph, Box::new(then_statements));
                    if let Some(then_id) = then_statement_ids.first() {
                        graph.add_edge(node_id, *then_id, LogicScriptStatementGraphEdge::IfThen);
                    }
                    if then_block_exits.is_empty() {
                        if let Some(last_then_id) = then_statement_ids.last() {
                            block_exit_node_ids.push(*last_then_id);
                        }
                    } else {
                        block_exit_node_ids.extend(&then_block_exits);
                    }

                    let else_statements = if_statement
                        .else_statements
                        .iter()
                        .map(|stmt| stmt.as_ref());
                    let (else_statement_ids, else_block_exits) =
                        add_statements_to_graph(graph, Box::new(else_statements));
                    if let Some(else_id) = else_statement_ids.first() {
                        graph.add_edge(node_id, *else_id, LogicScriptStatementGraphEdge::IfElse);
                    }
                    if else_block_exits.is_empty() {
                        if let Some(last_else_id) = else_statement_ids.last() {
                            block_exit_node_ids.push(*last_else_id);
                        }
                    } else {
                        block_exit_node_ids.extend(&else_block_exits);
                    }
                }
                _ => {}
            }

            node_id
        })
        .collect();

    (added_statements, block_exit_node_ids)
}

pub struct LogicScriptStatementGraph {
    pub graph: DiGraph<LogicScriptStatement<ParsedLogicArgument>, LogicScriptStatementGraphEdge>,
    pub root_id: NodeIndex,
    pub identifiers: IdentifierMap,
    pub node_ids_by_label: HashMap<String, NodeIndex>,
}

impl LogicScriptStatementGraph {
    pub fn try_from_statements(
        statements: &[LogicScriptStatement<ParsedLogicArgument>],
        identifiers: IdentifierMap,
    ) -> Result<Self, LogicScriptCodeGenerationError> {
        let mut graph = DiGraph::new();
        let (statement_ids, _) = add_statements_to_graph(&mut graph, Box::new(statements.iter()));
        let root_id = *statement_ids.first().unwrap();

        let all_node_ids = Dfs::new(&graph, root_id).iter(&graph).collect::<Vec<_>>();

        let node_ids_by_label: HashMap<String, NodeIndex> = all_node_ids
            .iter()
            .filter_map(|node_id| {
                let Some(LogicScriptStatement::Label(label)) = graph.node_weight(*node_id) else {
                    return None;
                };

                Some((label.label.clone(), *node_id))
            })
            .collect();

        for node_id in all_node_ids.iter() {
            let Some(statement) = graph.node_weight(*node_id) else {
                continue;
            };

            let LogicScriptStatement::CommandCall(command_call) = statement else {
                continue;
            };

            if command_call.command_name != "goto" {
                continue;
            }

            let Some(target_label) = statement.get_goto_target_label() else {
                return Err(LogicScriptCodeGenerationError::GotoWithNoTarget(
                    statement.clone(),
                ));
            };

            let Some(target_node_id) = node_ids_by_label.get(target_label) else {
                continue;
            };

            graph.add_edge(
                *node_id,
                *target_node_id,
                LogicScriptStatementGraphEdge::GotoTarget,
            );
        }

        Ok(LogicScriptStatementGraph {
            graph,
            root_id,
            identifiers,
            node_ids_by_label,
        })
    }

    pub fn to_statements(
        &self,
    ) -> Result<Vec<LogicScriptStatement<ParsedLogicArgument>>, LogicScriptCodeGenerationError>
    {
        type StackItem = (NodeIndex, LogicScriptStatement<ParsedLogicArgument>);
        let mut stack = VecDeque::<StackItem>::new();
        let traversal_filter = EdgeFiltered::from_fn(&self.graph, |edge| {
            *edge.weight() != LogicScriptStatementGraphEdge::GotoTarget
        });
        let next_filter = EdgeFiltered::from_fn(&self.graph, |edge| {
            *edge.weight() == LogicScriptStatementGraphEdge::Next
        });
        let mut dfs = DfsPostOrder::new(&traversal_filter, self.root_id);

        while let Some(node_id) = dfs.next(&traversal_filter) {
            let statement = self.node_to_statement(node_id)?;

            let statement = match statement {
                LogicScriptStatement::IfStatement(if_statement) => {
                    let then_node_id = self.graph.directed_neighbor_node_id_of_type(
                        node_id,
                        Direction::Outgoing,
                        LogicScriptStatementGraphEdge::IfThen,
                    );
                    let else_node_id = self.graph.directed_neighbor_node_id_of_type(
                        node_id,
                        Direction::Outgoing,
                        LogicScriptStatementGraphEdge::IfElse,
                    );

                    let mut then_statements = vec![];
                    let mut else_statements = vec![];
                    let mut other_statements = VecDeque::new();

                    for (subclause_node_id, subclause_statement) in stack.into_iter() {
                        if let Some(then_node_id) = then_node_id
                            && has_path_connecting(
                                &next_filter,
                                then_node_id,
                                subclause_node_id,
                                None,
                            )
                        {
                            then_statements.push(Box::new(subclause_statement));
                        } else if let Some(else_node_id) = else_node_id
                            && has_path_connecting(
                                &next_filter,
                                else_node_id,
                                subclause_node_id,
                                None,
                            )
                        {
                            else_statements.push(Box::new(subclause_statement));
                        } else {
                            other_statements.push_back((subclause_node_id, subclause_statement));
                        }
                    }

                    stack = other_statements;
                    LogicScriptStatement::IfStatement(LogicScriptIfStatement {
                        conditions: if_statement.conditions,
                        if_keyword: if_statement.if_keyword,
                        else_keyword: if_statement.else_keyword,
                        then_statements,
                        else_statements,
                    })
                }
                _ => statement,
            };

            stack.push_front((node_id, statement));
        }

        Ok(stack
            .into_iter()
            .map(|(_node_id, statement)| statement)
            .collect())
    }

    fn node_to_statement(
        &self,
        node_id: NodeIndex,
    ) -> Result<LogicScriptStatement<ParsedLogicArgument>, LogicScriptCodeGenerationError> {
        let Some(statement) = self.graph.node_weight(node_id) else {
            return Err(LogicScriptCodeGenerationError::StatementGraphNodeNotFound(
                node_id,
            ));
        };

        match statement {
            LogicScriptStatement::CommandCall(command_call) => {
                if command_call.command_name == "goto" {
                    let Some(target_node_id) = self.graph.directed_neighbor_node_id_of_type(
                        node_id,
                        Direction::Outgoing,
                        LogicScriptStatementGraphEdge::GotoTarget,
                    ) else {
                        eprintln!(
                            "{:?}",
                            self.graph
                                .edges_directed(node_id, Direction::Outgoing)
                                .collect::<Vec<_>>()
                        );
                        return Err(LogicScriptCodeGenerationError::GotoWithNoTarget(
                            statement.clone(),
                        ));
                    };

                    let Some(target_statement) = self.graph.node_weight(target_node_id) else {
                        return Err(LogicScriptCodeGenerationError::StatementGraphNodeNotFound(
                            target_node_id,
                        ));
                    };

                    let LogicScriptStatement::Label(label) = target_statement else {
                        return Err(LogicScriptCodeGenerationError::JumpToUnlabeledStatement(
                            target_node_id,
                            None,
                        ));
                    };
                    let target_argument = ParsedLogicArgument::Identifier(LogicIdentifier {
                        name: label.label.clone(),
                    });

                    Ok(LogicScriptStatement::CommandCall(LogicScriptCommandCall {
                        command_name: "goto".to_string(),
                        argument_list: vec![target_argument],
                    }))
                } else {
                    Ok(statement.clone())
                }
            }
            LogicScriptStatement::IfStatement(if_statement) => {
                Ok(LogicScriptStatement::IfStatement(LogicScriptIfStatement {
                    conditions: if_statement.conditions.clone(),
                    if_keyword: if_statement.if_keyword.clone(),
                    else_keyword: if_statement.else_keyword.clone(),
                    then_statements: vec![],
                    else_statements: vec![],
                }))
            }
            _ => Ok(statement.clone()),
        }
    }
}

impl Optimizable<DiGraph<LogicScriptStatement<ParsedLogicArgument>, LogicScriptStatementGraphEdge>>
    for LogicScriptStatementGraph
{
    fn get_graph(
        &self,
    ) -> &DiGraph<LogicScriptStatement<ParsedLogicArgument>, LogicScriptStatementGraphEdge> {
        &self.graph
    }

    fn get_graph_mut(
        &mut self,
    ) -> &mut DiGraph<LogicScriptStatement<ParsedLogicArgument>, LogicScriptStatementGraphEdge>
    {
        &mut self.graph
    }

    fn root_id(&self) -> NodeIndex {
        self.root_id
    }

    fn optimization_visitors(
        &self,
    ) -> Vec<
        Box<
            dyn OptimizationVisitor<
                DiGraph<LogicScriptStatement<ParsedLogicArgument>, LogicScriptStatementGraphEdge>,
            >,
        >,
    > {
        vec![
            Box::new(RemoveUnusedLabels::new(&self)),
            Box::new(remove_redundant_jumps),
            Box::new(remove_empty_then_with_else),
        ]
    }
}

pub struct RemoveUnusedLabels {
    used_labels: HashSet<String>,
}

impl RemoveUnusedLabels {
    pub fn new(statement_graph: &LogicScriptStatementGraph) -> Self {
        let used_labels = Dfs::new(&statement_graph.graph, statement_graph.root_id)
            .iter(&statement_graph.graph)
            .filter_map(|node_id| {
                let node = statement_graph.graph.node_weight(node_id);
                node.and_then(|node| node.get_goto_target_label())
            })
            .cloned()
            .collect();

        Self { used_labels }
    }
}

impl
    OptimizationVisitor<
        DiGraph<LogicScriptStatement<ParsedLogicArgument>, LogicScriptStatementGraphEdge>,
    > for RemoveUnusedLabels
{
    fn visit(
        &mut self,
        graph: &mut DiGraph<
            LogicScriptStatement<ParsedLogicArgument>,
            LogicScriptStatementGraphEdge,
        >,
        node_id: NodeIndex,
    ) -> bool {
        let Some(LogicScriptStatement::Label(label)) = graph.node_weight(node_id) else {
            return false;
        };

        if self.used_labels.contains(&label.label) {
            return false;
        }

        let Some(next_id) = graph
            .directed_neighbor_node_id_of_type(
                node_id,
                Direction::Outgoing,
                LogicScriptStatementGraphEdge::Next,
            )
            .or_else(|| {
                graph.directed_neighbor_node_id_of_type(
                    node_id,
                    Direction::Outgoing,
                    LogicScriptStatementGraphEdge::BlockExit,
                )
            })
        else {
            return false;
        };

        graph.remove_node_preserving_edges(node_id, next_id);
        true
    }
}

pub fn remove_redundant_jumps(
    graph: &mut DiGraph<LogicScriptStatement<ParsedLogicArgument>, LogicScriptStatementGraphEdge>,
    node_id: NodeIndex,
) -> bool {
    let Some(statement) = graph.node_weight(node_id) else {
        return false;
    };

    let Some(target_label) = statement.get_goto_target_label() else {
        return false;
    };

    let Some(next_id) = graph
        .directed_neighbor_node_id_of_type(
            node_id,
            Direction::Outgoing,
            LogicScriptStatementGraphEdge::Next,
        )
        .or_else(|| {
            graph.directed_neighbor_node_id_of_type(
                node_id,
                Direction::Outgoing,
                LogicScriptStatementGraphEdge::BlockExit,
            )
        })
    else {
        return false;
    };

    let Some(next_statement) = graph.node_weight(next_id) else {
        return false;
    };

    let is_redundant_jump = match next_statement {
        LogicScriptStatement::Label(label) => label.label == *target_label,
        _ => next_statement.get_goto_target_label() == Some(target_label),
    };

    if is_redundant_jump {
        graph.remove_node_preserving_edges(node_id, next_id);
        true
    } else {
        false
    }
}

pub fn remove_empty_then_with_else(
    graph: &mut DiGraph<LogicScriptStatement<ParsedLogicArgument>, LogicScriptStatementGraphEdge>,
    node_id: NodeIndex,
) -> bool {
    let Some(LogicScriptStatement::IfStatement(statement)) = graph.node_weight_mut(node_id) else {
        return false;
    };

    if statement.then_statements.is_empty() && statement.else_statements.len() > 0 {
        let inverse_conditions = LogicBooleanExpression::NotExpression(LogicNotExpression {
            expression: Box::new(statement.conditions.clone()),
        });
        statement.conditions = inverse_conditions;
        statement.then_statements = statement.else_statements.clone();
        statement.else_statements = vec![];

        let outgoing_edges = graph
            .edges_directed(node_id, Direction::Outgoing)
            .map(|edge| (edge.id(), edge.target(), edge.weight().to_owned()))
            .collect::<Vec<_>>();
        for (edge_id, target_id, weight) in outgoing_edges {
            let inverse_weight = match weight {
                LogicScriptStatementGraphEdge::IfThen => {
                    Some(LogicScriptStatementGraphEdge::IfElse)
                }
                LogicScriptStatementGraphEdge::IfElse => {
                    Some(LogicScriptStatementGraphEdge::IfThen)
                }
                _ => None,
            };
            if let Some(inverse_weight) = inverse_weight {
                graph.remove_edge(edge_id);
                graph.add_edge(node_id, target_id, inverse_weight);
            }
        }

        true
    } else {
        false
    }
}

#[cfg(feature = "dot")]
impl LogicScriptStatementGraph {
    pub fn to_dot(&self, context: &LogicScriptCodeGenerationContext) -> String {
        use petgraph::dot::{Config, Dot};

        format!(
            "{:?}",
            Dot::with_attr_getters(
                &self.graph,
                &[Config::NodeNoLabel],
                &|_graph_ref, _edge_ref| "".to_string(),
                &|_graph_ref, (_node_id, statement)| { statement.node_attrs(context) }
            )
        )
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        logic::{
            asm::expressions::ParsedLogicArgument,
            logic_script::{
                codegen::{
                    context::LogicScriptCodeGenerationContext,
                    program_generator::LogicScriptProgramGenerator,
                    statement_graph::LogicScriptStatementGraph,
                },
                identifiers::IdentifierMap,
                statements::LogicScriptStatement,
            },
        },
        project::Project,
        resources::ResourceType,
        test_data::uriquest,
    };

    use similar_asserts::assert_eq;

    fn statement_graph_comparison(
        project: &Project,
        logic_resource_number: u16,
    ) -> anyhow::Result<(
        Vec<LogicScriptStatement<ParsedLogicArgument>>,
        Vec<LogicScriptStatement<ParsedLogicArgument>>,
    )> {
        let logic = project.decode_logic(logic_resource_number)?;
        let word_list = project.decode_word_list()?;
        let context = LogicScriptCodeGenerationContext::try_from_program(&logic, &word_list)?;
        let generator = LogicScriptProgramGenerator::new(&context);
        let statements = generator.generate_statements()?;
        let statement_graph =
            LogicScriptStatementGraph::try_from_statements(&statements, IdentifierMap::builtins())?;
        let generated_statements = statement_graph.to_statements()?;

        Ok((statements, generated_statements))
    }

    macro_rules! logic_smoke_test {
        ($test_name: ident, $resource_number: literal) => {
            #[test]
            fn $test_name() {
                let project = uriquest();
                let (statements, generated_statements) =
                    statement_graph_comparison(&project, $resource_number).unwrap();

                assert_eq!(statements, generated_statements);
            }
        };
    }

    logic_smoke_test!(logic_0_test, 0);
    logic_smoke_test!(logic_13_test, 13);
    logic_smoke_test!(logic_93_test, 93);

    #[test]
    fn comprehensive_smoke_test() {
        let project = uriquest();

        let mut resource_numbers = project
            .resource_collection()
            .dirs
            .resource_numbers(ResourceType::LOGIC)
            .collect::<Vec<_>>();
        resource_numbers.sort();

        let failed = resource_numbers
            .into_iter()
            .filter_map(|resource_number| {
                let result = statement_graph_comparison(&project, resource_number);
                result
                    .and_then(|(statements, generated_statements)| {
                        if statements == generated_statements {
                            Ok((statements, generated_statements))
                        } else {
                            Err(anyhow::Error::msg(format!(
                                "LOGIC {} generated statements did not match",
                                resource_number
                            )))
                        }
                    })
                    .err()
                    .map(|err| format!("LOGIC {}: {}", resource_number, err))
            })
            .collect::<Vec<_>>();

        if !failed.is_empty() {
            assert!(false, "{:#?}", failed);
        }
    }
}

use std::collections::{HashMap, HashSet};
#[cfg(feature = "dot")]
use std::fmt::Debug;

use petgraph::{
    Direction,
    graph::{DiGraph, NodeIndex},
    visit::{Dfs, EdgeRef, Walker},
};

#[cfg(feature = "dot")]
use crate::logic::logic_script::codegen::context::LogicScriptCodeGenerationContext;
use crate::logic::{
    analysis::{
        dominator_tree::DominationAnalysis,
        optimization::{
            DirectedNeighborEdgeUtils, Optimizable, OptimizationVisitor, RemoveNodePreservingEdges,
        },
    },
    asm::expressions::{
        LogicArgument, LogicBooleanExpression, LogicIdentifier, LogicNotExpression,
        ParsedLogicArgument,
    },
    logic_script::{
        codegen::errors::LogicScriptCodeGenerationError,
        identifiers::IdentifierMap,
        statements::{LogicScriptCommandCall, LogicScriptIfStatement, LogicScriptStatement},
    },
};
#[cfg(test)]
use crate::test_utils::write_and_edit;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum LogicScriptStatementGraphEdge {
    Next,
    IfThen,
    IfElse,
}

fn add_statements_to_graph<'a, Arg: LogicArgument + Clone + 'a>(
    graph: &mut DiGraph<LogicScriptStatement<Arg>, LogicScriptStatementGraphEdge>,
    statements: Box<dyn Iterator<Item = &'a LogicScriptStatement<Arg>> + 'a>,
) -> (Vec<NodeIndex>, Vec<NodeIndex>) {
    let mut prev_node_ids: Vec<NodeIndex> = vec![];
    let added_statements: Vec<_> = statements
        .map(|statement| {
            let node_id = graph.add_node(statement.clone());

            for prev_node_id in prev_node_ids.iter() {
                graph.add_edge(*prev_node_id, node_id, LogicScriptStatementGraphEdge::Next);
            }
            prev_node_ids.clear();
            prev_node_ids.push(node_id);

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
                            prev_node_ids.push(*last_then_id);
                        }
                    } else {
                        prev_node_ids.extend(&then_block_exits);
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
                            prev_node_ids.push(*last_else_id);
                        }
                    } else {
                        prev_node_ids.extend(&else_block_exits);
                    }
                }
                _ => {}
            }

            node_id
        })
        .collect();

    (added_statements, prev_node_ids)
}

pub struct LogicScriptStatementGraph {
    pub graph: DiGraph<LogicScriptStatement<ParsedLogicArgument>, LogicScriptStatementGraphEdge>,
    pub root_id: NodeIndex,
    pub identifiers: IdentifierMap,
    pub node_ids_by_label: HashMap<String, NodeIndex>,
}

impl LogicScriptStatementGraph {
    pub fn from_statements(
        statements: &[LogicScriptStatement<ParsedLogicArgument>],
        identifiers: IdentifierMap,
    ) -> Self {
        let mut graph = DiGraph::new();
        let (statement_ids, _) = add_statements_to_graph(&mut graph, Box::new(statements.iter()));
        let root_id = *statement_ids.first().unwrap();

        let node_ids_by_label: HashMap<String, NodeIndex> = Dfs::new(&graph, root_id)
            .iter(&graph)
            .filter_map(|node_id| {
                let Some(LogicScriptStatement::Label(label)) = graph.node_weight(node_id) else {
                    return None;
                };

                Some((label.label.clone(), node_id))
            })
            .collect();

        let mut connect_gotos_dfs = Dfs::new(&graph, root_id);
        while let Some(node_id) = connect_gotos_dfs.next(&graph) {
            let Some(statement) = graph.node_weight(node_id) else {
                continue;
            };

            let Some(target_label) = statement.get_goto_target_label() else {
                continue;
            };

            let Some(target_node_id) = node_ids_by_label.get(target_label) else {
                continue;
            };

            if !graph.contains_edge(node_id, *target_node_id) {
                graph.add_edge(
                    node_id,
                    *target_node_id,
                    LogicScriptStatementGraphEdge::Next,
                );
            }
        }

        LogicScriptStatementGraph {
            graph,
            root_id,
            identifiers,
            node_ids_by_label,
        }
    }

    pub fn to_statements(
        &self,
    ) -> Result<Vec<LogicScriptStatement<ParsedLogicArgument>>, LogicScriptCodeGenerationError>
    {
        let domination_analysis = DominationAnalysis::from_graph(&self.graph, self.root_id);

        let mut statements = vec![];
        let mut current_node_id = self.root_id;

        loop {
            statements.push(self.node_to_statement(current_node_id, &domination_analysis)?);

            if let Some(post_dominator) =
                domination_analysis.immediate_post_dominator(current_node_id)
            {
                current_node_id = post_dominator;
            } else {
                break;
            }
        }

        Ok(statements)
    }

    fn node_to_statement(
        &self,
        node_id: NodeIndex,
        domination_analysis: &DominationAnalysis,
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
                        LogicScriptStatementGraphEdge::Next,
                    ) else {
                        return Err(LogicScriptCodeGenerationError::GotoWithNoTarget(
                            statement.clone(),
                        ));
                    };

                    let Some(target_statement) = self.graph.node_weight(target_node_id) else {
                        return Err(LogicScriptCodeGenerationError::GotoWithNoTarget(
                            statement.clone(),
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
                let generate_subclause_statements = |subclause_node_id| {
                    let mut statements = vec![];
                    let mut current_node_id = subclause_node_id;
                    while !domination_analysis.post_dominates(current_node_id, node_id) {
                        statements
                            .push(self.node_to_statement(current_node_id, domination_analysis)?);
                        let Some(next_node_id) =
                            domination_analysis.immediate_post_dominator(current_node_id)
                        else {
                            break;
                        };
                        current_node_id = next_node_id;
                    }
                    Ok::<_, LogicScriptCodeGenerationError>(statements)
                };

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

                let then_statements = then_node_id
                    .map(|then_node_id| generate_subclause_statements(then_node_id))
                    .transpose()?
                    .unwrap_or_default();
                let else_statements = else_node_id
                    .map(|else_node_id| generate_subclause_statements(else_node_id))
                    .transpose()?
                    .unwrap_or_default();

                Ok(LogicScriptStatement::IfStatement(LogicScriptIfStatement {
                    conditions: if_statement.conditions.clone(),
                    if_keyword: if_statement.if_keyword.clone(),
                    else_keyword: if_statement.else_keyword.clone(),
                    then_statements: then_statements.into_iter().map(Box::new).collect(),
                    else_statements: else_statements.into_iter().map(Box::new).collect(),
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

        let Some(next_id) = graph.directed_neighbor_node_id_of_type(
            node_id,
            Direction::Outgoing,
            LogicScriptStatementGraphEdge::Next,
        ) else {
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

    let Some(next_id) = graph.directed_neighbor_node_id_of_type(
        node_id,
        Direction::Outgoing,
        LogicScriptStatementGraphEdge::Next,
    ) else {
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
    use std::os::macos::raw::stat;

    use crate::{
        agi_version::AGIVersion,
        logic::{
            LogicProgram,
            analysis::dominator_tree::DominationAnalysis,
            logic_script::{
                codegen::{
                    context::LogicScriptCodeGenerationContext,
                    program_generator::LogicScriptProgramGenerator,
                    statement_graph::LogicScriptStatementGraph,
                },
                identifiers::IdentifierMap,
            },
        },
        resources::{ResourceType, decode::Decode, file_provider::FileProvider},
        test_data::{uriquest_dir, uriquest_resources},
        test_utils::write_and_edit,
        word_list::WordList,
    };

    use pretty_assertions::assert_eq;

    #[test]
    fn test_to_statements() {
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

        let generator = LogicScriptProgramGenerator::new(&context);

        let statements = generator
            .generate_statements()
            .expect("Failed to generate logic script statements");

        let statement_graph =
            LogicScriptStatementGraph::from_statements(&statements, IdentifierMap::builtins());

        // write_and_edit(".dot", &statement_graph.to_dot(&context));
        let domination_analysis =
            DominationAnalysis::from_graph(&statement_graph.graph, statement_graph.root_id);

        write_and_edit(
            ".dot",
            &domination_analysis
                .post_dominators_to_dot(&statement_graph.graph, &|_node_index, node| {
                    node.node_attrs(&context)
                }),
        );

        let generated_statements = statement_graph
            .to_statements()
            .expect("Failed to generate statements from graph");

        assert_eq!(statements, generated_statements);
    }
}

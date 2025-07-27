use std::collections::{HashSet, VecDeque};
#[cfg(feature = "dot")]
use std::fmt::Debug;

use petgraph::{
    Direction,
    algo::has_path_connecting,
    graph::NodeIndex,
    prelude::StableDiGraph,
    visit::{Dfs, DfsPostOrder, EdgeFiltered, EdgeRef, Walker},
};

#[cfg(feature = "dot")]
use crate::logic::logic_script::codegen::context::LogicScriptCodeGenerationContext;
use crate::logic::{
    analysis::{
        dominator_tree::{DominationAnalysis, DominatorTree},
        optimization::{
            DirectedNeighborEdgeUtils, Optimizable, OptimizationPass, OptimizationResult,
            OptimizationVisitor, RemoveNodePreservingEdges,
        },
    },
    asm::expressions::{
        LogicBooleanExpression, LogicIdentifier, LogicNotExpression, ParsedLogicArgument,
    },
    logic_script::{
        codegen::{
            errors::LogicScriptCodeGenerationError,
            node_label_map::{GetOrInsertLabelResult, LabeledNode, NodeLabelMap},
        },
        identifiers::IdentifierMap,
        statements::{
            LogicScriptCommandCall, LogicScriptIfStatement, LogicScriptStatement,
            LogicScriptStatementBody,
        },
    },
};

#[derive(Debug, Clone)]
pub struct LogicScriptStatementGraphNode {
    statement: LogicScriptStatement<ParsedLogicArgument>,
}

impl LogicScriptStatementGraphNode {
    pub fn statement(&self) -> &LogicScriptStatement<ParsedLogicArgument> {
        &self.statement
    }

    pub fn get_goto_target_label(&self) -> Option<&String> {
        self.statement.get_goto_target_label()
    }

    pub fn node_attrs(&self, context: &LogicScriptCodeGenerationContext) -> String {
        self.statement.dot_node_attrs(context)
    }
}

impl From<LogicScriptStatement<ParsedLogicArgument>> for LogicScriptStatementGraphNode {
    fn from(value: LogicScriptStatement<ParsedLogicArgument>) -> Self {
        Self { statement: value }
    }
}

impl LabeledNode for LogicScriptStatementGraphNode {
    fn label(&self) -> Option<&str> {
        self.statement.label()
    }

    fn set_label(&mut self, label: Option<&str>) {
        self.statement.set_label(label);
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogicScriptStatementGraphEdge {
    Next,
    GotoTarget,
    IfThen,
    IfElse,
    BlockExit,
}

fn add_statements_to_graph<'a>(
    graph: &mut StableDiGraph<LogicScriptStatementGraphNode, LogicScriptStatementGraphEdge>,
    statements: Box<dyn Iterator<Item = &'a LogicScriptStatement<ParsedLogicArgument>> + 'a>,
) -> (Vec<NodeIndex>, Vec<NodeIndex>) {
    let mut prev_node_id: Option<NodeIndex> = None;
    let mut block_exit_node_ids: Vec<NodeIndex> = vec![];
    let added_statements: Vec<_> = statements
        .map(|statement| {
            let node_id = graph.add_node(statement.clone().into());

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

            match &statement.body {
                LogicScriptStatementBody::IfStatement(if_statement) => {
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

#[derive(Debug, Clone)]
pub struct LogicScriptStatementGraph {
    pub graph: StableDiGraph<LogicScriptStatementGraphNode, LogicScriptStatementGraphEdge>,
    pub root_id: NodeIndex,
    pub identifiers: IdentifierMap,
    pub label_map: NodeLabelMap,
}

impl LogicScriptStatementGraph {
    pub fn try_from_statements(
        statements: &[LogicScriptStatement<ParsedLogicArgument>],
        identifiers: IdentifierMap,
    ) -> Result<Self, LogicScriptCodeGenerationError> {
        let mut graph = StableDiGraph::new();
        let (statement_ids, _) = add_statements_to_graph(&mut graph, Box::new(statements.iter()));
        let root_id = *statement_ids.first().unwrap();

        let all_node_ids = Dfs::new(&graph, root_id).iter(&graph).collect::<Vec<_>>();

        let label_map = NodeLabelMap::new(&graph, root_id);

        for node_id in all_node_ids.iter() {
            let Some(node) = graph.node_weight(*node_id) else {
                continue;
            };

            let LogicScriptStatementBody::CommandCall(command_call) = &node.statement().body else {
                continue;
            };

            if command_call.command_name != "goto" {
                continue;
            }

            let Some(target_label) = node.get_goto_target_label() else {
                return Err(LogicScriptCodeGenerationError::GotoWithNoTarget(
                    node.statement().clone(),
                ));
            };

            let Some(target_node_id) = label_map.get_node_id_for_label(target_label) else {
                continue;
            };

            graph.add_edge(
                *node_id,
                target_node_id,
                LogicScriptStatementGraphEdge::GotoTarget,
            );
        }

        Ok(LogicScriptStatementGraph {
            graph,
            root_id,
            identifiers,
            label_map,
        })
    }

    pub fn to_statements(
        &mut self,
    ) -> Result<Vec<LogicScriptStatement<ParsedLogicArgument>>, LogicScriptCodeGenerationError>
    {
        type StackItem = (NodeIndex, LogicScriptStatement<ParsedLogicArgument>);
        let mut stack = VecDeque::<StackItem>::new();
        let traversal_filter = EdgeFiltered::from_fn(&self.graph, |edge| {
            *edge.weight() != LogicScriptStatementGraphEdge::GotoTarget
        });
        let inward_filter = EdgeFiltered::from_fn(&traversal_filter, |edge| {
            *edge.weight() != LogicScriptStatementGraphEdge::BlockExit
        });
        let no_then_filter = EdgeFiltered::from_fn(&traversal_filter, |edge| {
            *edge.weight() != LogicScriptStatementGraphEdge::IfThen
        });
        let no_else_filter = EdgeFiltered::from_fn(&traversal_filter, |edge| {
            *edge.weight() != LogicScriptStatementGraphEdge::IfElse
        });
        let dominator_tree = DominatorTree::from_graph(&traversal_filter, self.root_id);
        let mut dfs_post_order = DfsPostOrder::new(&traversal_filter, self.root_id);
        let mut inserted_labels: Vec<(NodeIndex, String)> = vec![];

        while let Some(node_id) = dfs_post_order.next(&traversal_filter) {
            let generated_statement = self.node_to_statement(node_id)?;

            let statement = match &generated_statement.body {
                LogicScriptStatementBody::IfStatement(if_statement) => {
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

                    let is_then_statement = |subclause_node_id| {
                        if let Some(then_node_id) = then_node_id
                            && has_path_connecting(
                                &inward_filter,
                                then_node_id,
                                subclause_node_id,
                                None,
                            )
                            && !has_path_connecting(
                                &no_then_filter,
                                node_id,
                                subclause_node_id,
                                None,
                            )
                        {
                            dominator_tree.dominates(node_id, subclause_node_id)
                        } else {
                            false
                        }
                    };
                    let is_else_statement = |subclause_node_id| {
                        if let Some(else_node_id) = else_node_id
                            && has_path_connecting(
                                &inward_filter,
                                else_node_id,
                                subclause_node_id,
                                None,
                            )
                            && !has_path_connecting(
                                &no_else_filter,
                                node_id,
                                subclause_node_id,
                                None,
                            )
                        {
                            dominator_tree.dominates(node_id, subclause_node_id)
                        } else {
                            false
                        }
                    };
                    let mut append_goto_if_needed = |subclause_statements: &mut Vec<(
                        Option<NodeIndex>,
                        Box<LogicScriptStatement<ParsedLogicArgument>>,
                    )>| {
                        if let Some((Some(last_statement_id), _)) = subclause_statements.last() {
                            let exit_ids = self
                                .graph
                                .neighbors_directed(*last_statement_id, Direction::Outgoing)
                                .collect::<Vec<_>>();

                            let dominates_exit = exit_ids
                                .iter()
                                .all(|next_id| dominator_tree.dominates(node_id, *next_id));

                            if !dominates_exit {
                                let target_id = *exit_ids.first().unwrap();
                                let target_label =
                                    self.label_map.get_or_insert_label_for_node_id(target_id);

                                eprintln!(
                                    "node_id: {node_id:?} exit_ids: {exit_ids:?} target_label: {target_label:?}"
                                );

                                match target_label {
                                    GetOrInsertLabelResult::Inserted(ref label) => {
                                        inserted_labels.push((target_id, label.clone()))
                                    }
                                    _ => {}
                                }

                                subclause_statements.push((
                                    None,
                                    Box::new(LogicScriptStatement::new(
                                        LogicScriptStatementBody::CommandCall(
                                            LogicScriptCommandCall {
                                                command_name: "goto".to_string(),
                                                argument_list: vec![
                                                    ParsedLogicArgument::Identifier(
                                                        LogicIdentifier {
                                                            name: target_label.label().to_string(),
                                                        },
                                                    ),
                                                ],
                                            },
                                        ),
                                        None,
                                    )),
                                ));
                            }
                        }
                    };

                    for (subclause_node_id, subclause_statement) in stack.into_iter() {
                        if is_then_statement(subclause_node_id) {
                            then_statements
                                .push((Some(subclause_node_id), Box::new(subclause_statement)));
                        } else if is_else_statement(subclause_node_id) {
                            else_statements
                                .push((Some(subclause_node_id), Box::new(subclause_statement)));
                        } else {
                            other_statements.push_back((subclause_node_id, subclause_statement));
                        }
                    }

                    append_goto_if_needed(&mut then_statements);
                    append_goto_if_needed(&mut else_statements);

                    stack = other_statements;
                    LogicScriptStatement::new(
                        LogicScriptStatementBody::IfStatement(LogicScriptIfStatement {
                            conditions: if_statement.conditions.clone(),
                            if_keyword: if_statement.if_keyword.clone(),
                            else_keyword: if_statement.else_keyword.clone(),
                            then_statements: then_statements
                                .into_iter()
                                .map(|(_, statement)| statement)
                                .collect(),
                            else_statements: else_statements
                                .into_iter()
                                .map(|(_, statement)| statement)
                                .collect(),
                        }),
                        self.label_map
                            .get_label_for_node_id(node_id)
                            .map(|l| l.to_string()),
                    )
                }
                _ => generated_statement,
            };

            stack.push_front((node_id, statement));
        }

        Ok(stack
            .into_iter()
            .map(|(node_id, statement)| {
                LogicScriptStatement::new(
                    statement.body,
                    self.label_map
                        .get_label_for_node_id(node_id)
                        .map(|l| l.to_string()),
                )
            })
            .collect())
    }

    fn node_to_statement(
        &self,
        node_id: NodeIndex,
    ) -> Result<LogicScriptStatement<ParsedLogicArgument>, LogicScriptCodeGenerationError> {
        let Some(node) = self.graph.node_weight(node_id) else {
            return Err(LogicScriptCodeGenerationError::StatementGraphNodeNotFound(
                node_id,
            ));
        };

        let statement = node.statement();

        let generated_statement = match &node.statement().body {
            LogicScriptStatementBody::CommandCall(command_call) => {
                if command_call.command_name == "goto" {
                    let Some(target_node_id) = self.graph.directed_neighbor_node_id_of_type(
                        node_id,
                        Direction::Outgoing,
                        LogicScriptStatementGraphEdge::GotoTarget,
                    ) else {
                        return Err(LogicScriptCodeGenerationError::GotoWithNoTarget(
                            statement.clone(),
                        ));
                    };

                    let Some(target_statement) = self.graph.node_weight(target_node_id) else {
                        return Err(LogicScriptCodeGenerationError::StatementGraphNodeNotFound(
                            target_node_id,
                        ));
                    };

                    let Some(label) = &target_statement.statement().label else {
                        return Err(LogicScriptCodeGenerationError::JumpToUnlabeledStatement(
                            target_node_id,
                            None,
                        ));
                    };
                    let target_argument = ParsedLogicArgument::Identifier(LogicIdentifier {
                        name: label.clone(),
                    });

                    LogicScriptStatement::new(
                        LogicScriptStatementBody::CommandCall(LogicScriptCommandCall {
                            command_name: "goto".to_string(),
                            argument_list: vec![target_argument],
                        }),
                        None,
                    )
                } else {
                    statement.clone()
                }
            }
            LogicScriptStatementBody::IfStatement(if_statement) => LogicScriptStatement::new(
                LogicScriptStatementBody::IfStatement(LogicScriptIfStatement {
                    conditions: if_statement.conditions.clone(),
                    if_keyword: if_statement.if_keyword.clone(),
                    else_keyword: if_statement.else_keyword.clone(),
                    then_statements: vec![],
                    else_statements: vec![],
                }),
                self.label_map
                    .get_label_for_node_id(node_id)
                    .map(|l| l.to_string()),
            ),
            _ => statement.clone(),
        };

        Ok(generated_statement)
    }
}

impl Optimizable<StableDiGraph<LogicScriptStatementGraphNode, LogicScriptStatementGraphEdge>>
    for LogicScriptStatementGraph
{
    fn get_graph(
        &self,
    ) -> &StableDiGraph<LogicScriptStatementGraphNode, LogicScriptStatementGraphEdge> {
        &self.graph
    }

    fn get_graph_mut(
        &mut self,
    ) -> &mut StableDiGraph<LogicScriptStatementGraphNode, LogicScriptStatementGraphEdge> {
        &mut self.graph
    }

    fn root_id(&self) -> NodeIndex {
        self.root_id
    }

    fn optimization_passes(
        &self,
    ) -> Vec<
        Box<
            dyn OptimizationPass<
                StableDiGraph<LogicScriptStatementGraphNode, LogicScriptStatementGraphEdge>,
            >,
        >,
    > {
        vec![
            Box::new(RemoveUnusedLabelsPass::new(&self)),
            Box::new(RemoveRedundantJumpsPass::new(&self)),
            Box::new(remove_empty_then_with_else),
            Box::new(transform_post_dominating_else_to_next),
        ]
    }
}

pub struct RemoveUnusedLabelsPass {
    used_labels: HashSet<String>,
}

impl RemoveUnusedLabelsPass {
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
    OptimizationVisitor<StableDiGraph<LogicScriptStatementGraphNode, LogicScriptStatementGraphEdge>>
    for RemoveUnusedLabelsPass
{
    fn visit(
        &mut self,
        graph: &mut StableDiGraph<LogicScriptStatementGraphNode, LogicScriptStatementGraphEdge>,
        node_id: NodeIndex,
    ) -> OptimizationResult {
        let Some(label) = graph.node_weight(node_id).and_then(|n| n.label()) else {
            return OptimizationResult::Unchanged;
        };

        if self.used_labels.contains(label) {
            return OptimizationResult::Unchanged;
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
            .or_else(|| {
                graph.directed_neighbor_node_id_of_type(
                    node_id,
                    Direction::Outgoing,
                    LogicScriptStatementGraphEdge::GotoTarget,
                )
            })
        else {
            return OptimizationResult::Unchanged;
        };

        let incoming_edges = graph.incoming_edge_data(node_id);
        for (edge_id, source_id, weight) in incoming_edges {
            if !graph.contains_edge(source_id, next_id) {
                graph.add_edge(source_id, next_id, weight);
            }
            graph.remove_edge(edge_id);
        }

        graph.remove_node(node_id);
        OptimizationResult::Changed
    }
}

pub struct RemoveRedundantJumpsPass {
    domination_analysis: DominationAnalysis,
}

impl RemoveRedundantJumpsPass {
    pub fn new(statement_graph: &LogicScriptStatementGraph) -> Self {
        let domination_analysis =
            DominationAnalysis::from_graph(&statement_graph.graph, statement_graph.root_id);

        Self {
            domination_analysis,
        }
    }
}

impl
    OptimizationVisitor<StableDiGraph<LogicScriptStatementGraphNode, LogicScriptStatementGraphEdge>>
    for RemoveRedundantJumpsPass
{
    fn visit(
        &mut self,
        graph: &mut StableDiGraph<LogicScriptStatementGraphNode, LogicScriptStatementGraphEdge>,
        node_id: NodeIndex,
    ) -> OptimizationResult {
        let Some(target_id) = graph.directed_neighbor_node_id_of_type(
            node_id,
            Direction::Outgoing,
            LogicScriptStatementGraphEdge::GotoTarget,
        ) else {
            return OptimizationResult::Unchanged;
        };

        let prev_edges = graph.incoming_edge_data(node_id);

        let is_redundant_jump = prev_edges
            .iter()
            .all(|(_, prev_id, _)| self.domination_analysis.post_dominates(target_id, *prev_id));

        if is_redundant_jump {
            for (edge_id, source_id, weight) in prev_edges {
                if !graph.contains_edge(source_id, target_id) {
                    graph.add_edge(source_id, target_id, weight);
                }
                graph.remove_edge(edge_id);
            }

            graph.remove_node(node_id);
            OptimizationResult::Changed
        } else {
            OptimizationResult::Unchanged
        }
    }
}

pub fn transform_post_dominating_else_to_next(
    graph: &mut StableDiGraph<LogicScriptStatementGraphNode, LogicScriptStatementGraphEdge>,
    node_id: NodeIndex,
) -> OptimizationResult {
    let Some(LogicScriptStatementBody::IfStatement(_)) = graph
        .node_weight_mut(node_id)
        .map(|n| &mut n.statement.body)
    else {
        return OptimizationResult::Unchanged;
    };

    let Some(then_node_id) = graph.directed_neighbor_node_id_of_type(
        node_id,
        Direction::Outgoing,
        LogicScriptStatementGraphEdge::IfThen,
    ) else {
        return OptimizationResult::Unchanged;
    };

    let Some(else_node_id) = graph.directed_neighbor_node_id_of_type(
        node_id,
        Direction::Outgoing,
        LogicScriptStatementGraphEdge::IfElse,
    ) else {
        return OptimizationResult::Unchanged;
    };

    let is_post_dominator = {
        let next_filter = EdgeFiltered::from_fn(graph as &StableDiGraph<_, _>, |edge| {
            *edge.weight() == LogicScriptStatementGraphEdge::Next
        });
        has_path_connecting(&next_filter, then_node_id, else_node_id, None)
    };

    if is_post_dominator {
        graph.update_edge(node_id, else_node_id, LogicScriptStatementGraphEdge::Next);
        OptimizationResult::Changed
    } else {
        OptimizationResult::Unchanged
    }
}

pub fn remove_empty_then_with_else(
    graph: &mut StableDiGraph<LogicScriptStatementGraphNode, LogicScriptStatementGraphEdge>,
    node_id: NodeIndex,
) -> OptimizationResult {
    let Some(LogicScriptStatementBody::IfStatement(statement)) = graph
        .node_weight_mut(node_id)
        .map(|n| &mut n.statement.body)
    else {
        return OptimizationResult::Unchanged;
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

        OptimizationResult::Changed
    } else {
        OptimizationResult::Unchanged
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
            analysis::optimization::{DirectedNeighborEdgeUtils, OptimizationPass},
            asm::expressions::{LogicIdentifier, ParsedLogicArgument},
            logic_script::{
                codegen::{
                    context::LogicScriptCodeGenerationContext,
                    node_label_map::NodeLabelMap,
                    program_generator::LogicScriptProgramGenerator,
                    statement_graph::{
                        LogicScriptStatementGraph, LogicScriptStatementGraphEdge,
                        LogicScriptStatementGraphNode, RemoveRedundantJumpsPass,
                    },
                },
                identifiers::IdentifierMap,
                statements::{
                    LogicScriptCommandCall, LogicScriptStatement, LogicScriptStatementBody,
                },
            },
        },
        project::Project,
        resources::ResourceType,
        test_data::uriquest,
    };

    use petgraph::{Direction, prelude::StableDiGraph};
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
        let mut statement_graph =
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
    logic_smoke_test!(logic_99_test, 99);
    logic_smoke_test!(logic_100_test, 100);

    #[test]
    fn test_remove_redundant_jumps() {
        let mut graph = StableDiGraph::new();
        let first_statement_id = graph.add_node(LogicScriptStatementGraphNode::from(
            LogicScriptStatement::new(
                LogicScriptStatementBody::CommandCall(LogicScriptCommandCall {
                    command_name: "increment".to_string(),
                    argument_list: vec![ParsedLogicArgument::Identifier(LogicIdentifier {
                        name: "v1".to_string(),
                    })],
                }),
                None,
            ),
        ));
        let goto_id = graph.add_node(LogicScriptStatementGraphNode::from(
            LogicScriptStatement::new(
                LogicScriptStatementBody::CommandCall(LogicScriptCommandCall {
                    command_name: "goto".to_string(),
                    argument_list: vec![ParsedLogicArgument::Identifier(LogicIdentifier {
                        name: "JumpTarget".to_string(),
                    })],
                }),
                None,
            ),
        ));
        let target_statement_id = graph.add_node(LogicScriptStatementGraphNode::from(
            LogicScriptStatement::new(
                LogicScriptStatementBody::CommandCall(LogicScriptCommandCall {
                    command_name: "increment".to_string(),
                    argument_list: vec![ParsedLogicArgument::Identifier(LogicIdentifier {
                        name: "v2".to_string(),
                    })],
                }),
                Some("JumpTarget".to_string()),
            ),
        ));
        graph.add_edge(
            first_statement_id,
            goto_id,
            LogicScriptStatementGraphEdge::Next,
        );
        graph.add_edge(
            goto_id,
            target_statement_id,
            LogicScriptStatementGraphEdge::GotoTarget,
        );
        let mut statement_graph = LogicScriptStatementGraph {
            label_map: NodeLabelMap::new(&graph, first_statement_id),
            graph,
            root_id: first_statement_id,
            identifiers: IdentifierMap::builtins(),
        };
        let mut pass = RemoveRedundantJumpsPass::new(&statement_graph);

        assert!(
            pass.run(&mut statement_graph.graph, statement_graph.root_id)
                .is_changed(),
            "Optimization did not remove a node"
        );
        assert_eq!(2, statement_graph.graph.node_count());
        assert_eq!(1, statement_graph.graph.edge_count());
        assert_eq!(
            Some(target_statement_id),
            statement_graph.graph.directed_neighbor_node_id_of_type(
                first_statement_id,
                Direction::Outgoing,
                LogicScriptStatementGraphEdge::Next
            )
        );
    }

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

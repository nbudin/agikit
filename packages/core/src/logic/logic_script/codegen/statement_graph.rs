use std::collections::{HashMap, HashSet, VecDeque};
#[cfg(feature = "dot")]
use std::fmt::Debug;

use petgraph::{
    Direction,
    algo::has_path_connecting,
    graph::NodeIndex,
    prelude::{StableDiGraph, StableGraph},
    visit::{Dfs, DfsPostOrder, EdgeFiltered, EdgeRef, Walker},
};

#[cfg(feature = "dot")]
use crate::logic::asm::codegen::AsmCodeGenerationContext;
use crate::logic::{
    analysis::{
        dominator_tree::DominationAnalysis,
        optimization::{
            DirectedNeighborEdgeUtils, Optimizable, OptimizationPass, OptimizationResult,
            OptimizationVisitor, RemoveNodePreservingEdges,
        },
    },
    asm::expressions::{
        AsParsedLogicArgument, LogicArgument, LogicBooleanExpression, LogicIdentifier,
        LogicNotExpression, ParsedLogicArgument,
    },
    logic_script::{
        codegen::{
            errors::LogicScriptCodeGenerationError,
            node_label_map::{LabeledNode, NodeLabelMap},
        },
        identifiers::IdentifierMap,
        statements::{
            LogicScriptCommandCall, LogicScriptIfStatement, LogicScriptStatement,
            LogicScriptStatementBody, StatementWithOrWithoutLocation,
        },
    },
};

pub trait LogicScriptStatementGraphNode: Clone + Debug + LabeledNode {
    type SubclauseStatement: AsRef<Self>;

    fn get_goto_target_label(&self) -> Option<&str>;
    fn if_subclauses(&self) -> Option<(&[Self::SubclauseStatement], &[Self::SubclauseStatement])>;
    /// Returns true if this is an if statement with an else keyword, even if the
    /// else_statements list is empty (as happens after unrolling).
    fn has_else_keyword(&self) -> bool;
    #[cfg(feature = "dot")]
    fn node_attrs(&self, context: &AsmCodeGenerationContext) -> String;
}

impl<Arg: LogicArgument + AsParsedLogicArgument + Clone + Debug> LogicScriptStatementGraphNode
    for LogicScriptStatement<Arg>
{
    type SubclauseStatement = StatementWithOrWithoutLocation<Arg>;

    fn get_goto_target_label(&self) -> Option<&str> {
        LogicScriptStatement::get_goto_target_label(&self).map(|l| l.as_str())
    }

    fn if_subclauses(&self) -> Option<(&[Self::SubclauseStatement], &[Self::SubclauseStatement])> {
        match &self.body {
            LogicScriptStatementBody::IfStatement(body) => Some((
                body.then_statements.as_slice(),
                body.else_statements.as_slice(),
            )),
            _ => None,
        }
    }

    fn has_else_keyword(&self) -> bool {
        match &self.body {
            LogicScriptStatementBody::IfStatement(body) => body.else_keyword.is_some(),
            _ => false,
        }
    }

    #[cfg(feature = "dot")]
    fn node_attrs(&self, context: &AsmCodeGenerationContext) -> String {
        self.dot_node_attrs(context)
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

fn add_statements_to_graph<'a, N: LogicScriptStatementGraphNode>(
    graph: &mut StableDiGraph<N, LogicScriptStatementGraphEdge>,
    statements: Box<dyn Iterator<Item = &'a N> + 'a>,
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

            if let Some((then_statements, else_statements)) = statement.if_subclauses() {
                let then_statements = then_statements.iter().map(|stmt| stmt.as_ref());
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

                let else_statements = else_statements.iter().map(|stmt| stmt.as_ref());
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

            node_id
        })
        .collect();

    (added_statements, block_exit_node_ids)
}

#[derive(Debug, Clone)]
pub struct LogicScriptStatementGraph<N: LogicScriptStatementGraphNode> {
    pub graph: StableDiGraph<N, LogicScriptStatementGraphEdge>,
    pub root_id: NodeIndex,
    pub identifiers: IdentifierMap,
    pub label_map: NodeLabelMap,
    /// Labels that were initially referenced by goto statements.  These labels
    /// should be preserved even after the gotos that reference them are removed
    /// by optimization passes, because they serve as meaningful address markers
    /// in the decompiled output.
    initial_goto_target_labels: HashSet<String>,
    /// Maps if-statement node IDs to their convergence point node IDs.
    /// For if-else statements where the else was unrolled, the convergence
    /// point is where both branches reconverge. This is computed before
    /// optimization (when convergence gotos still exist) so it survives
    /// after the gotos are removed by optimization passes.
    convergence_points: HashMap<NodeIndex, NodeIndex>,
}

impl<N: LogicScriptStatementGraphNode> LogicScriptStatementGraph<N> {
    pub fn try_from_statements(
        statements: &[N],
        identifiers: IdentifierMap,
    ) -> Result<Self, LogicScriptCodeGenerationError> {
        let mut graph: StableGraph<N, LogicScriptStatementGraphEdge> = StableDiGraph::new();
        let (statement_ids, _) = add_statements_to_graph(&mut graph, Box::new(statements.iter()));
        let root_id = *statement_ids.first().unwrap();

        let all_node_ids = Dfs::new(&graph, root_id).iter(&graph).collect::<Vec<_>>();

        let label_map = NodeLabelMap::new(&graph, root_id);

        for node_id in all_node_ids.iter() {
            let Some(node) = graph.node_weight(*node_id) else {
                continue;
            };

            let Some(target_label) = node.get_goto_target_label() else {
                continue;
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

        // Compute convergence points for if statements with unrolled else clauses.
        // For each if node with else_keyword but no IfElse edge, find the convergence
        // goto in the then clause and record its GotoTarget as the convergence point.
        let mut convergence_points = HashMap::new();
        let mut convergence_goto_node_ids = HashSet::new();
        for node_id in Dfs::new(&graph, root_id).iter(&graph) {
            let Some(node) = graph.node_weight(node_id) else {
                continue;
            };
            if !node.has_else_keyword() {
                continue;
            }
            // Check if this if has no IfElse edge (unrolled else)
            let has_if_else_edge = graph
                .edges_directed(node_id, Direction::Outgoing)
                .any(|e| *e.weight() == LogicScriptStatementGraphEdge::IfElse);
            if has_if_else_edge {
                continue;
            }
            // Find the IfThen successor and walk through the then clause
            let Some(then_id) = graph
                .edges_directed(node_id, Direction::Outgoing)
                .find(|e| *e.weight() == LogicScriptStatementGraphEdge::IfThen)
                .map(|e| e.target())
            else {
                continue;
            };
            // Walk through the then clause following Next edges to find a convergence goto
            let mut current = Some(then_id);
            while let Some(curr_id) = current {
                if let Some(target) = graph
                    .edges_directed(curr_id, Direction::Outgoing)
                    .find(|e| *e.weight() == LogicScriptStatementGraphEdge::GotoTarget)
                    .map(|e| e.target())
                {
                    convergence_points.insert(node_id, target);
                    convergence_goto_node_ids.insert(curr_id);
                    break;
                }
                current = graph
                    .edges_directed(curr_id, Direction::Outgoing)
                    .find(|e| *e.weight() == LogicScriptStatementGraphEdge::Next)
                    .map(|e| e.target());
            }
        }

        // For each unrolled else clause, collect ALL node IDs reachable from
        // the if's Next successor up to (but not including) the convergence
        // point. We follow Next, IfThen, IfElse, and BlockExit edges to
        // capture nodes at all nesting levels within the else clause body.
        let mut else_clause_node_sets: Vec<HashSet<NodeIndex>> = Vec::new();
        for (if_node_id, convergence_id) in convergence_points.iter() {
            let Some(next_id) = graph
                .edges_directed(*if_node_id, Direction::Outgoing)
                .find(|e| *e.weight() == LogicScriptStatementGraphEdge::Next)
                .map(|e| e.target())
            else {
                continue;
            };

            let mut else_body_nodes = HashSet::new();
            let mut to_visit = VecDeque::from([next_id]);
            while let Some(curr_id) = to_visit.pop_front() {
                if curr_id == *convergence_id || else_body_nodes.contains(&curr_id) {
                    continue;
                }
                else_body_nodes.insert(curr_id);
                for edge in graph.edges_directed(curr_id, Direction::Outgoing) {
                    match edge.weight() {
                        LogicScriptStatementGraphEdge::Next
                        | LogicScriptStatementGraphEdge::IfThen
                        | LogicScriptStatementGraphEdge::IfElse
                        | LogicScriptStatementGraphEdge::BlockExit => {
                            to_visit.push_back(edge.target());
                        }
                        _ => {}
                    }
                }
            }
            else_clause_node_sets.push(else_body_nodes);
        }

        // Collect ALL goto target node IDs (both convergence and non-convergence).
        let all_goto_target_node_ids: HashSet<NodeIndex> = Dfs::new(&graph, root_id)
            .iter(&graph)
            .filter_map(|node_id| {
                let node = graph.node_weight(node_id)?;
                if node.get_goto_target_label().is_none() {
                    return None;
                }
                graph
                    .edges_directed(node_id, Direction::Outgoing)
                    .find(|e| *e.weight() == LogicScriptStatementGraphEdge::GotoTarget)
                    .map(|e| e.target())
            })
            .collect();

        // Preserve goto target labels ONLY when they are inside an else clause
        // body. Labels inside else clause bodies serve as meaningful section
        // markers in the decompiled output, even after the gotos that reference
        // them are removed by optimization. Labels outside else clause bodies
        // will be naturally preserved by used_labels if their gotos survive
        // optimization, or removed if their gotos are optimized away.
        let initial_goto_target_labels: HashSet<String> = all_goto_target_node_ids
            .iter()
            .filter(|target_node_id| {
                else_clause_node_sets
                    .iter()
                    .any(|node_set| node_set.contains(target_node_id))
            })
            .filter_map(|target_node_id| {
                let node = graph.node_weight(*target_node_id)?;
                let label = node.label()?;
                Some(label.to_string())
            })
            .collect();

        Ok(LogicScriptStatementGraph {
            graph,
            root_id,
            identifiers,
            label_map,
            initial_goto_target_labels,
            convergence_points,
        })
    }
}

impl<Arg: LogicArgument + AsParsedLogicArgument + Clone + Debug>
    LogicScriptStatementGraph<LogicScriptStatement<Arg>>
{
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
        let domination_analysis = DominationAnalysis::from_graph(&traversal_filter, self.root_id);

        let mut dfs_post_order = DfsPostOrder::new(&traversal_filter, self.root_id);

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
                    let next_node_id = self.graph.directed_neighbor_node_id_of_type(
                        node_id,
                        Direction::Outgoing,
                        LogicScriptStatementGraphEdge::Next,
                    );
                    // When there's an else_keyword but no IfElse edge, the else was
                    // unrolled by the program generator. The Next successor is the
                    // start of the implicit else clause. The program generator only
                    // sets else_keyword for real else clauses (not simple ifs where
                    // the then branch falls through to the continuation).
                    //
                    // The convergence point (where both branches reconverge) was
                    // computed before optimization from the convergence goto that
                    // the program generator places at the end of the then clause.
                    let (implicit_else_node_id, convergence_node_id) = if else_node_id.is_none()
                        && if_statement.else_keyword.is_some()
                        && next_node_id.is_some()
                    {
                        (next_node_id, self.convergence_points.get(&node_id).copied())
                    } else {
                        (None, None)
                    };

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
                            domination_analysis.dominates(node_id, subclause_node_id)
                        } else {
                            false
                        }
                    };
                    let is_else_statement = |subclause_node_id| {
                        // Explicit IfElse edge case
                        if let Some(else_node_id) = else_node_id
                            && has_path_connecting(
                                &inward_filter,
                                else_node_id,
                                subclause_node_id,
                                None,
                            )
                            && !then_node_id.map_or(false, |tn| {
                                has_path_connecting(&inward_filter, tn, subclause_node_id, None)
                            })
                        {
                            return domination_analysis.dominates(node_id, subclause_node_id);
                        }
                        // Implicit else case: no IfElse edge, but else_keyword is present.
                        // A node is an implicit else statement if:
                        // 1. Reachable from the Next successor (implicit else start) via inward edges
                        // 2. NOT the convergence point or reachable from it via inward edges
                        //    (the convergence point is where both branches reconverge)
                        // 3. Dominated by the if node
                        if let Some(implicit_else_start) = implicit_else_node_id
                            && has_path_connecting(
                                &inward_filter,
                                implicit_else_start,
                                subclause_node_id,
                                None,
                            )
                            && !convergence_node_id.map_or(false, |cn| {
                                cn == subclause_node_id
                                    || has_path_connecting(
                                        &inward_filter,
                                        cn,
                                        subclause_node_id,
                                        None,
                                    )
                            })
                        {
                            return domination_analysis.dominates(node_id, subclause_node_id);
                        }
                        false
                    };
                    for (subclause_node_id, subclause_statement) in stack.into_iter() {
                        if is_then_statement(subclause_node_id) {
                            then_statements.push(Box::new(subclause_statement));
                        } else if is_else_statement(subclause_node_id) {
                            else_statements.push(Box::new(subclause_statement));
                        } else {
                            other_statements.push_back((subclause_node_id, subclause_statement));
                        }
                    }

                    stack = other_statements;
                    LogicScriptStatement::new(
                        LogicScriptStatementBody::IfStatement(LogicScriptIfStatement {
                            conditions: if_statement.conditions.clone(),
                            if_keyword: if_statement.if_keyword.clone(),
                            else_keyword: if_statement.else_keyword.clone(),
                            then_statements: then_statements
                                .into_iter()
                                .map(|statement| {
                                    StatementWithOrWithoutLocation::WithoutLocation(
                                        statement.as_ref().clone(),
                                    )
                                })
                                .collect(),
                            else_statements: else_statements
                                .into_iter()
                                .map(|statement| {
                                    StatementWithOrWithoutLocation::WithoutLocation(
                                        statement.as_ref().clone(),
                                    )
                                })
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
            .map(|(_node_id, statement)| statement)
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

        let statement = node.clone();

        let generated_statement = match &node.body {
            LogicScriptStatementBody::CommandCall(command_call) => {
                if command_call.command_name == "goto" {
                    let Some(target_node_id) = self.graph.directed_neighbor_node_id_of_type(
                        node_id,
                        Direction::Outgoing,
                        LogicScriptStatementGraphEdge::GotoTarget,
                    ) else {
                        return Err(LogicScriptCodeGenerationError::GotoWithNoTarget(
                            statement.to_parsed(),
                        ));
                    };

                    let Some(target_statement) = self.graph.node_weight(target_node_id) else {
                        return Err(LogicScriptCodeGenerationError::StatementGraphNodeNotFound(
                            target_node_id,
                        ));
                    };

                    let Some(label) = &target_statement.label else {
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
                    statement.to_parsed()
                }
            }
            LogicScriptStatementBody::IfStatement(if_statement) => LogicScriptStatement::new(
                LogicScriptStatementBody::IfStatement(LogicScriptIfStatement {
                    conditions: if_statement.conditions.to_parsed(),
                    if_keyword: if_statement.if_keyword.clone(),
                    else_keyword: if_statement.else_keyword.clone(),
                    then_statements: vec![],
                    else_statements: vec![],
                }),
                self.label_map
                    .get_label_for_node_id(node_id)
                    .map(|l| l.to_string()),
            ),
            _ => statement.to_parsed(),
        };

        Ok(generated_statement)
    }
}

impl<Arg: LogicArgument + AsParsedLogicArgument + Clone + Debug + 'static>
    Optimizable<StableDiGraph<LogicScriptStatement<Arg>, LogicScriptStatementGraphEdge>>
    for LogicScriptStatementGraph<LogicScriptStatement<Arg>>
{
    fn get_graph(
        &self,
    ) -> &StableDiGraph<LogicScriptStatement<Arg>, LogicScriptStatementGraphEdge> {
        &self.graph
    }

    fn get_graph_mut(
        &mut self,
    ) -> &mut StableDiGraph<LogicScriptStatement<Arg>, LogicScriptStatementGraphEdge> {
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
                StableDiGraph<LogicScriptStatement<Arg>, LogicScriptStatementGraphEdge>,
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

    fn run_optimization_passes_once(
        &mut self,
        _passes: &mut [Box<
            dyn OptimizationPass<
                StableDiGraph<LogicScriptStatement<Arg>, LogicScriptStatementGraphEdge>,
            >,
        >],
    ) -> OptimizationResult {
        let mut remove_redundant_jumps = RemoveRedundantJumpsPass::new(&self);

        // Run RemoveRedundantJumpsPass first so that gotos are removed before
        // we check which labels are still referenced.
        let mut result = remove_redundant_jumps.run(&mut self.graph, self.root_id);

        let mut remove_unused_labels = RemoveUnusedLabelsPass::new(&self);
        result = result.or(&remove_unused_labels.run(&mut self.graph, self.root_id));
        for node_id in remove_unused_labels.removed_label_node_ids {
            self.label_map.remove_label_for_node_id(node_id);
        }

        result = result.or(&remove_empty_then_with_else(&mut self.graph, self.root_id));
        result = result.or(&transform_post_dominating_else_to_next(
            &mut self.graph,
            self.root_id,
        ));

        result
    }
}

pub struct RemoveUnusedLabelsPass {
    used_labels: HashSet<String>,
    initial_goto_target_labels: HashSet<String>,
    removed_label_node_ids: Vec<NodeIndex>,
}

impl RemoveUnusedLabelsPass {
    pub fn new<N: LogicScriptStatementGraphNode>(
        statement_graph: &LogicScriptStatementGraph<N>,
    ) -> Self {
        let used_labels = Dfs::new(&statement_graph.graph, statement_graph.root_id)
            .iter(&statement_graph.graph)
            .filter_map(|node_id| {
                let node = statement_graph.graph.node_weight(node_id);
                node.and_then(|node| node.get_goto_target_label().map(|l| l.to_string()))
            })
            .collect();

        Self {
            used_labels,
            initial_goto_target_labels: statement_graph.initial_goto_target_labels.clone(),
            removed_label_node_ids: vec![],
        }
    }
}

impl<N: LogicScriptStatementGraphNode>
    OptimizationVisitor<StableDiGraph<N, LogicScriptStatementGraphEdge>>
    for RemoveUnusedLabelsPass
{
    fn visit(
        &mut self,
        graph: &mut StableDiGraph<N, LogicScriptStatementGraphEdge>,
        node_id: NodeIndex,
    ) -> OptimizationResult {
        let Some(node) = graph.node_weight(node_id) else {
            return OptimizationResult::Unchanged;
        };

        let Some(label) = node.label() else {
            return OptimizationResult::Unchanged;
        };

        // Keep labels that are currently referenced by gotos
        if self.used_labels.contains(label) {
            return OptimizationResult::Unchanged;
        }

        // Keep labels that were initially goto targets. These labels mark
        // meaningful bytecode addresses even after the gotos referencing them
        // are removed by optimization passes.
        if self.initial_goto_target_labels.contains(label) {
            return OptimizationResult::Unchanged;
        }

        let Some(node) = graph.node_weight_mut(node_id) else {
            return OptimizationResult::Unchanged;
        };

        node.set_label(None);
        self.removed_label_node_ids.push(node_id);
        OptimizationResult::Changed
    }
}

pub struct RemoveRedundantJumpsPass {
    domination_analysis: DominationAnalysis,
}

impl RemoveRedundantJumpsPass {
    pub fn new<N: LogicScriptStatementGraphNode>(
        statement_graph: &LogicScriptStatementGraph<N>,
    ) -> Self {
        let domination_analysis =
            DominationAnalysis::from_graph(&statement_graph.graph, statement_graph.root_id);

        Self {
            domination_analysis,
        }
    }
}

impl<N: LogicScriptStatementGraphNode>
    OptimizationVisitor<StableDiGraph<N, LogicScriptStatementGraphEdge>>
    for RemoveRedundantJumpsPass
{
    fn visit(
        &mut self,
        graph: &mut StableDiGraph<N, LogicScriptStatementGraphEdge>,
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

        // A goto is NOT redundant if its target dominates any of its predecessors,
        // because that means the goto implements a back-edge (loop). Removing it would
        // either create a self-loop or lose the loop structure.
        let is_back_edge = prev_edges
            .iter()
            .any(|(_, prev_id, _)| self.domination_analysis.dominates(target_id, *prev_id));

        let is_redundant_jump = !is_back_edge
            && prev_edges.iter().all(|(_, prev_id, _)| {
                self.domination_analysis.post_dominates(target_id, *prev_id)
            });

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

pub fn transform_post_dominating_else_to_next<Arg: LogicArgument + AsParsedLogicArgument>(
    graph: &mut StableDiGraph<LogicScriptStatement<Arg>, LogicScriptStatementGraphEdge>,
    node_id: NodeIndex,
) -> OptimizationResult {
    let Some(LogicScriptStatementBody::IfStatement(_)) =
        graph.node_weight_mut(node_id).map(|n| &mut n.body)
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

pub fn remove_empty_then_with_else<Arg: LogicArgument + AsParsedLogicArgument + Clone>(
    graph: &mut StableDiGraph<LogicScriptStatement<Arg>, LogicScriptStatementGraphEdge>,
    node_id: NodeIndex,
) -> OptimizationResult {
    let Some(LogicScriptStatementBody::IfStatement(statement)) =
        graph.node_weight_mut(node_id).map(|n| &mut n.body)
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
impl<N: LogicScriptStatementGraphNode> LogicScriptStatementGraph<N> {
    pub fn to_dot(&self, context: &AsmCodeGenerationContext) -> String {
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
                        RemoveRedundantJumpsPass,
                    },
                },
                identifiers::IdentifierMap,
                statements::{
                    LogicScriptCommandCall, LogicScriptStatement, LogicScriptStatementBody,
                },
            },
        },
        project::Project,
        resources::{ResourceType, file_provider::FileProvider},
        test_data::uriquest,
    };

    use std::collections::{HashMap, HashSet};

    use petgraph::{Direction, prelude::StableDiGraph};
    use similar_asserts::assert_eq;

    /// Verify that the statement graph roundtrip is idempotent: running
    /// statements through the graph twice produces the same result.  The first
    /// pass may restructure unrolled else clauses into proper if/else blocks,
    /// so we cannot compare against the original program generator output.
    /// However, the second pass should be a no-op.
    fn statement_graph_idempotency_check<FP: FileProvider>(
        project: &Project<FP>,
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

        // First pass: build graph from program generator output and convert to statements.
        // This may restructure unrolled else clauses into proper if/else blocks.
        let mut graph1 =
            LogicScriptStatementGraph::try_from_statements(&statements, IdentifierMap::builtins())?;
        let pass1_statements = graph1.to_statements()?;

        // Second pass: build a new graph from the first pass output and convert again.
        // This should produce identical output, proving the transformation is stable.
        let mut graph2 = LogicScriptStatementGraph::try_from_statements(
            &pass1_statements,
            IdentifierMap::builtins(),
        )?;
        let pass2_statements = graph2.to_statements()?;

        Ok((pass1_statements, pass2_statements))
    }

    macro_rules! logic_smoke_test {
        ($test_name: ident, $resource_number: literal) => {
            #[test]
            fn $test_name() {
                let project = uriquest();
                let (pass1_statements, pass2_statements) =
                    statement_graph_idempotency_check(&project, $resource_number).unwrap();

                assert_eq!(pass1_statements, pass2_statements);
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
        let first_statement_id = graph.add_node(LogicScriptStatement::new(
            LogicScriptStatementBody::CommandCall(LogicScriptCommandCall {
                command_name: "increment".to_string(),
                argument_list: vec![ParsedLogicArgument::Identifier(LogicIdentifier {
                    name: "v1".to_string(),
                })],
            }),
            None,
        ));
        let goto_id = graph.add_node(LogicScriptStatement::new(
            LogicScriptStatementBody::CommandCall(LogicScriptCommandCall {
                command_name: "goto".to_string(),
                argument_list: vec![ParsedLogicArgument::Identifier(LogicIdentifier {
                    name: "JumpTarget".to_string(),
                })],
            }),
            None,
        ));
        let target_statement_id = graph.add_node(LogicScriptStatement::new(
            LogicScriptStatementBody::CommandCall(LogicScriptCommandCall {
                command_name: "increment".to_string(),
                argument_list: vec![ParsedLogicArgument::Identifier(LogicIdentifier {
                    name: "v2".to_string(),
                })],
            }),
            Some("JumpTarget".to_string()),
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
            initial_goto_target_labels: HashSet::new(),
            convergence_points: HashMap::new(),
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
            .lock()
            .unwrap()
            .dirs
            .resource_numbers(ResourceType::LOGIC)
            .collect::<Vec<_>>();
        resource_numbers.sort();

        let failed = resource_numbers
            .into_iter()
            .filter_map(|resource_number| {
                let result = statement_graph_idempotency_check(&project, resource_number);
                result
                    .and_then(|(pass1_statements, pass2_statements)| {
                        if pass1_statements == pass2_statements {
                            Ok((pass1_statements, pass2_statements))
                        } else {
                            Err(anyhow::Error::msg(format!(
                                "LOGIC {} statement graph roundtrip is not idempotent",
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

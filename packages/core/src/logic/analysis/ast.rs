use std::{collections::HashMap, fmt::Display};

use petgraph::{Direction, graph::NodeIndex, prelude::StableDiGraph, visit::EdgeRef};

#[cfg(feature = "dot")]
use crate::logic::asm::codegen::AsmCodeGenerationContext;
use crate::logic::{
    LogicCommand, LogicConditionClause, LogicInstruction,
    asm::{LogicLabel, codegen::generate_labels},
};

pub type LogicASTGraph = StableDiGraph<LogicASTNode, LogicASTEdge>;

#[derive(Debug, Clone)]
pub struct LogicASTNodeMetadata {
    pub instruction_address: Option<u16>,
}

#[derive(Debug, Clone)]
pub struct LogicCommandNode {
    pub command: LogicCommand,
    pub label: Option<LogicLabel>,
    pub metadata: LogicASTNodeMetadata,
}

#[derive(Debug, Clone)]
pub struct LogicIfNode {
    pub clauses: Vec<LogicConditionClause>,
    pub label: Option<LogicLabel>,
    pub metadata: LogicASTNodeMetadata,
}

#[derive(Debug, Clone)]
pub struct LogicGotoNode {
    pub label: Option<LogicLabel>,
    pub metadata: LogicASTNodeMetadata,
}

#[derive(Debug, Clone)]
pub enum LogicASTNode {
    Command(LogicCommandNode),
    If(LogicIfNode),
    Goto(LogicGotoNode),
}

impl LogicASTNode {
    pub fn label(&self) -> Option<&LogicLabel> {
        match self {
            LogicASTNode::Command(node) => node.label.as_ref(),
            LogicASTNode::If(node) => node.label.as_ref(),
            LogicASTNode::Goto(node) => node.label.as_ref(),
        }
    }

    pub fn metadata(&self) -> &LogicASTNodeMetadata {
        match self {
            LogicASTNode::Command(node) => &node.metadata,
            LogicASTNode::If(node) => &node.metadata,
            LogicASTNode::Goto(node) => &node.metadata,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LogicASTEdge {
    CommandToNext,
    IfThen,
    IfElse,
    GotoToTarget,
}

pub struct LogicAST {
    pub graph: StableDiGraph<LogicASTNode, LogicASTEdge>,
    pub root_node_id: NodeIndex,
    pub nodes_by_address: HashMap<u16, NodeIndex>,
}

impl AsRef<LogicASTGraph> for LogicAST {
    fn as_ref(&self) -> &LogicASTGraph {
        &self.graph
    }
}

impl LogicAST {
    pub fn from_instructions(
        instructions: &[LogicInstruction],
    ) -> Result<Self, DecompilationError> {
        let labels = generate_labels(instructions, &[])
            .into_iter()
            .map(|label| (label.address, label))
            .collect::<HashMap<_, _>>();

        let mut unresolved_nodes = instructions
            .iter()
            .map(UnresolvedLogicASTNode::from_instruction)
            .collect::<Vec<_>>();

        let mut graph = StableDiGraph::new();
        let mut nodes_by_address = HashMap::new();
        let root_node_id = resolve_nodes(
            &mut unresolved_nodes,
            0,
            &labels,
            &mut graph,
            &mut nodes_by_address,
        )?;

        Ok(Self {
            graph,
            root_node_id,
            nodes_by_address,
        })
    }

    pub fn root_node(&self) -> &LogicASTNode {
        self.graph
            .node_weight(self.root_node_id)
            .expect("Root node not found")
    }

    pub fn outgoing_neighbor_id_of_type(
        &self,
        node_id: NodeIndex,
        edge_type: LogicASTEdge,
    ) -> Option<NodeIndex> {
        self.graph
            .edges_directed(node_id, Direction::Outgoing)
            .find_map(|edge| {
                if edge.weight() == &edge_type {
                    Some(edge.target())
                } else {
                    None
                }
            })
    }

    pub fn next_node_id(&self, node_id: NodeIndex) -> Option<NodeIndex> {
        self.outgoing_neighbor_id_of_type(node_id, LogicASTEdge::CommandToNext)
    }

    pub fn then_node_id(&self, node_id: NodeIndex) -> Option<NodeIndex> {
        self.outgoing_neighbor_id_of_type(node_id, LogicASTEdge::IfThen)
    }

    pub fn else_node_id(&self, node_id: NodeIndex) -> Option<NodeIndex> {
        self.outgoing_neighbor_id_of_type(node_id, LogicASTEdge::IfElse)
    }

    pub fn goto_target_node_id(&self, node_id: NodeIndex) -> Option<NodeIndex> {
        self.outgoing_neighbor_id_of_type(node_id, LogicASTEdge::GotoToTarget)
    }
}

#[cfg(feature = "dot")]
impl LogicAST {
    pub fn to_dot(&self, context: &AsmCodeGenerationContext) -> String {
        use petgraph::dot::{Config, Dot};

        use crate::logic::asm::{codegen::GenerateLogicAsm, expressions::LogicBooleanExpression};

        format!(
            "{:?}",
            Dot::with_attr_getters(
                &self.graph,
                &[Config::NodeNoLabel],
                &|_graph_ref, _edge_ref| "".to_string(),
                &|_graph_ref, (_node_id, node_weight)| {
                    let shape = match node_weight {
                        LogicASTNode::Command(_) => "box",
                        LogicASTNode::Goto(_) => "invtriangle",
                        LogicASTNode::If(_) => "diamond",
                    };
                    let label = match node_weight {
                        LogicASTNode::Command(node) => node
                            .command
                            .generate_asm(context, &HashMap::new())
                            .expect("Failed to generate asm"),
                        LogicASTNode::Goto(_) => "goto".to_string(),
                        LogicASTNode::If(node) => format!(
                            "if ({})",
                            LogicBooleanExpression::from_clauses(&node.clauses, context)
                                .expect("Failed to generate boolean expression")
                                .generate_asm(context, &HashMap::new())
                                .expect("Failed to generate asm")
                        ),
                    };
                    format!(
                        "shape = {}, label = {}",
                        shape,
                        serde_json::to_string(&label).unwrap()
                    )
                }
            )
        )
    }
}

#[derive(Debug, Clone)]
pub enum DecompilationError {
    InvalidJump {
        target_address: u16,
        current_address: u16,
    },
}

impl Display for DecompilationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DecompilationError::InvalidJump {
                target_address,
                current_address,
            } => write!(
                f,
                "Invalid jump to {} at {}",
                target_address, current_address
            ),
        }
    }
}

#[derive(Debug, Clone)]
struct UnresolvedIfNode {
    address: u16,
    clauses: Vec<LogicConditionClause>,
    else_goto_address: u16,
}

#[derive(Debug, Clone)]
struct UnresolvedGotoNode {
    address: u16,
    jump_target_address: u16,
}

#[derive(Debug, Clone)]
enum UnresolvedLogicASTNode {
    Command(LogicCommand),
    If(UnresolvedIfNode),
    Goto(UnresolvedGotoNode),
}

impl UnresolvedLogicASTNode {
    fn from_instruction(instruction: &LogicInstruction) -> Self {
        match instruction {
            LogicInstruction::Command(logic_command) => Self::Command(logic_command.clone()),
            LogicInstruction::Condition(logic_condition) => Self::If(UnresolvedIfNode {
                address: logic_condition.address,
                clauses: logic_condition.clauses.clone(),
                else_goto_address: logic_condition.skip_address,
            }),
            LogicInstruction::Goto(logic_goto) => Self::Goto(UnresolvedGotoNode {
                address: logic_goto.address,
                jump_target_address: logic_goto.jump_address,
            }),
        }
    }

    fn address(&self) -> u16 {
        match self {
            UnresolvedLogicASTNode::Command(command) => command.address,
            UnresolvedLogicASTNode::If(if_node) => if_node.address,
            UnresolvedLogicASTNode::Goto(goto_node) => goto_node.address,
        }
    }
}

fn resolve_nodes(
    unresolved_nodes: &mut Vec<UnresolvedLogicASTNode>,
    current_node_index: usize,
    labels: &HashMap<u16, LogicLabel>,
    graph: &mut StableDiGraph<LogicASTNode, LogicASTEdge>,
    nodes_by_address: &mut HashMap<u16, NodeIndex>,
) -> Result<NodeIndex, DecompilationError> {
    let current_node = unresolved_nodes[current_node_index].clone();

    let existing_node = nodes_by_address.get(&current_node.address());
    if let Some(existing_node) = existing_node {
        return Ok(existing_node.clone());
    }

    match &current_node {
        UnresolvedLogicASTNode::Command(logic_command) => {
            let node_index = graph.add_node(LogicASTNode::Command(LogicCommandNode {
                command: logic_command.clone(),
                label: labels.get(&logic_command.address).cloned(),
                metadata: LogicASTNodeMetadata {
                    instruction_address: Some(logic_command.address),
                },
            }));
            nodes_by_address.insert(current_node.address(), node_index.clone());

            if logic_command.agi_command.name != "return"
                && current_node_index + 1 < unresolved_nodes.len()
            {
                let next_index = resolve_nodes(
                    unresolved_nodes,
                    current_node_index + 1,
                    labels,
                    graph,
                    nodes_by_address,
                )?;

                graph.add_edge(node_index, next_index, LogicASTEdge::CommandToNext);
            }

            Ok(node_index)
        }
        UnresolvedLogicASTNode::If(unresolved_if_node) => {
            // we're going to transform an AGI assembly conditional into something like:
            //
            // if (conditions) {
            //   stuffAfterTheIf
            // } else {
            //   goto(SkipTarget);
            // }
            //
            // which we can then optimize in later passes using control flow analysis
            let if_node_index = graph.add_node(LogicASTNode::If(LogicIfNode {
                clauses: unresolved_if_node.clauses.clone(),
                label: labels.get(&current_node.address()).cloned(),
                metadata: LogicASTNodeMetadata {
                    instruction_address: Some(unresolved_if_node.address),
                },
            }));
            nodes_by_address.insert(unresolved_if_node.address, if_node_index.clone());

            let goto_node_address = unresolved_nodes
                .iter()
                .map(|n| n.address())
                .max()
                .map(|max_address| max_address + 1)
                .unwrap_or(0);
            let goto_node_index = unresolved_nodes.len();

            // insert a virtual goto at the end of the code for the skip target
            let goto_node = UnresolvedLogicASTNode::Goto(UnresolvedGotoNode {
                address: goto_node_address,
                jump_target_address: unresolved_if_node.else_goto_address,
            });
            unresolved_nodes.push(goto_node);

            if current_node_index + 1 < unresolved_nodes.len() {
                let then_node_index = resolve_nodes(
                    unresolved_nodes,
                    current_node_index + 1,
                    labels,
                    graph,
                    nodes_by_address,
                )?;

                graph.add_edge(if_node_index, then_node_index, LogicASTEdge::IfThen);
            }

            let goto_node_index = resolve_nodes(
                unresolved_nodes,
                goto_node_index,
                labels,
                graph,
                nodes_by_address,
            )?;
            graph.add_edge(if_node_index, goto_node_index, LogicASTEdge::IfElse);

            Ok(if_node_index)
        }
        UnresolvedLogicASTNode::Goto(unresolved_goto_node) => {
            let target_index = nodes_by_address.get(&unresolved_goto_node.jump_target_address);

            let target_index = match target_index {
                Some(target) => target.clone(),
                None => {
                    let target_index = unresolved_nodes
                        .iter()
                        .position(|n| n.address() == unresolved_goto_node.jump_target_address)
                        .ok_or_else(|| DecompilationError::InvalidJump {
                            target_address: unresolved_goto_node.jump_target_address,
                            current_address: unresolved_goto_node.address,
                        })?;

                    resolve_nodes(
                        unresolved_nodes,
                        target_index,
                        labels,
                        graph,
                        nodes_by_address,
                    )?
                }
            };

            let goto_node_index = graph.add_node(LogicASTNode::Goto(LogicGotoNode {
                label: labels.get(&unresolved_goto_node.address).cloned(),
                metadata: LogicASTNodeMetadata {
                    instruction_address: Some(unresolved_goto_node.address),
                },
            }));
            nodes_by_address.insert(unresolved_goto_node.address, goto_node_index.clone());
            graph.add_edge(goto_node_index, target_index, LogicASTEdge::GotoToTarget);
            Ok(goto_node_index)
        }
    }
}

#[cfg(test)]
mod tests {

    use std::{fs::File, io::Write};

    use crate::{
        agi_version::AGIVersion,
        logic::{LogicProgram, analysis::ast::LogicAST, asm::codegen::AsmCodeGenerationContext},
        resources::{ResourceType, decode::Decode, file_provider::FileProvider},
        test_data::{uriquest, uriquest_dir},
        word_list::WordList,
    };

    #[test]
    fn test_build_ast() {
        let collection = uriquest();

        let logic_data = collection
            .read_resource_data(ResourceType::LOGIC, 0)
            .expect("Failed to read logic resource 0");
        let logic_program =
            LogicProgram::decode_from_bytes(&logic_data.data, &AGIVersion::new(2, 917))
                .expect("Failed to decode logic program");
        let word_list =
            WordList::decode_from_bytes(&uriquest_dir().read_file_bytes("WORDS.TOK").unwrap(), ())
                .unwrap();

        let ast = LogicAST::from_instructions(&logic_program.instructions)
            .expect("Failed to build AST from instructions");
        assert!(
            ast.root_node().metadata().instruction_address.is_some(),
            "AST should have a root node"
        );

        File::create("ast-debug.dot")
            .expect("Failed to open ast-debug.dot for writing")
            .write_fmt(format_args!(
                "{}",
                ast.to_dot(&AsmCodeGenerationContext {
                    messages: &logic_program.messages,
                    word_list: &word_list
                })
            ))
            .expect("Failed to write dot diagram");
    }
}

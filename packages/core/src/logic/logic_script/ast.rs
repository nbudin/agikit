use std::{collections::HashMap, fmt::Display};

use crate::logic::{
    asm::{codegen::generate_labels, LogicLabel},
    LogicCommand, LogicConditionClause, LogicInstruction,
};

pub type LogicASTNodeID = String;

#[derive(Debug, Clone)]
pub struct LogicASTNodeMetadata {
    pub instruction_address: Option<u16>,
}

#[derive(Debug, Clone)]
pub struct LogicCommandNode {
    pub command: LogicCommand,
    pub id: LogicASTNodeID,
    pub label: Option<LogicLabel>,
    pub next: Option<LogicASTNodeID>,
    pub metadata: LogicASTNodeMetadata,
}

#[derive(Debug, Clone)]
pub struct LogicIfNode {
    pub id: LogicASTNodeID,
    pub clauses: Vec<LogicConditionClause>,
    pub then: Option<LogicASTNodeID>,
    pub else_: Option<LogicASTNodeID>,
    pub label: Option<LogicLabel>,
    pub metadata: LogicASTNodeMetadata,
}

#[derive(Debug, Clone)]
pub struct LogicGotoNode {
    pub id: LogicASTNodeID,
    pub jump_target: LogicASTNodeID,
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
    pub fn id(&self) -> &LogicASTNodeID {
        match self {
            LogicASTNode::Command(node) => &node.id,
            LogicASTNode::If(node) => &node.id,
            LogicASTNode::Goto(node) => &node.id,
        }
    }

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

#[derive(Debug, Clone)]
pub struct LogicAST {
    root_node_id: LogicASTNodeID,
    nodes: HashMap<LogicASTNodeID, LogicASTNode>,
}

impl LogicAST {
    pub fn new(root_node_id: LogicASTNodeID, nodes: HashMap<LogicASTNodeID, LogicASTNode>) -> Self {
        Self {
            root_node_id,
            nodes,
        }
    }

    pub fn get_node(&self, id: &LogicASTNodeID) -> Option<&LogicASTNode> {
        self.nodes.get(id)
    }

    pub fn root_node(&self) -> &LogicASTNode {
        self.nodes.get(&self.root_node_id).unwrap()
    }

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

        let mut node_index = HashMap::new();
        let root_node_id = resolve_nodes(&mut unresolved_nodes, 0, &labels, &mut node_index)?;
        let nodes = node_index
            .into_iter()
            .map(|(_, node)| (node.id().clone(), node))
            .collect::<HashMap<_, _>>();

        Ok(LogicAST::new(root_node_id.clone(), nodes))
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
    node_index: &mut HashMap<u16, LogicASTNode>,
) -> Result<LogicASTNodeID, DecompilationError> {
    let current_node = unresolved_nodes[current_node_index].clone();

    let existing_node = node_index.get(&current_node.address());
    if let Some(existing_node) = existing_node {
        return Ok(existing_node.id().clone());
    }

    let node_id = format!("{}", current_node.address());
    match &current_node {
        UnresolvedLogicASTNode::Command(logic_command) => {
            node_index.insert(
                current_node.address(),
                LogicASTNode::Command(LogicCommandNode {
                    command: logic_command.clone(),
                    id: node_id.clone(),
                    label: labels.get(&logic_command.address).cloned(),
                    next: None,
                    metadata: LogicASTNodeMetadata {
                        instruction_address: Some(logic_command.address),
                    },
                }),
            );

            if logic_command.agi_command.name != "return"
                && current_node_index + 1 < unresolved_nodes.len()
            {
                let next =
                    resolve_nodes(unresolved_nodes, current_node_index + 1, labels, node_index)?;

                let node = node_index.get_mut(&current_node.address()).unwrap();
                if let LogicASTNode::Command(command_node) = node {
                    command_node.next = Some(next.clone());
                }
            }

            Ok(node_id)
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
            let if_node = LogicASTNode::If(LogicIfNode {
                id: node_id.clone(),
                clauses: unresolved_if_node.clauses.clone(),
                then: None,
                else_: None,
                label: labels.get(&current_node.address()).cloned(),
                metadata: LogicASTNodeMetadata {
                    instruction_address: Some(unresolved_if_node.address),
                },
            });
            node_index.insert(unresolved_if_node.address, if_node);

            let goto_node_address = unresolved_nodes
                .iter()
                .map(|n| n.address())
                .max()
                .map(|max_address| max_address + 1)
                .unwrap_or(0);
            let goto_node_index = unresolved_nodes.len();

            if current_node_index + 1 < unresolved_nodes.len() {
                let next_node_id =
                    resolve_nodes(unresolved_nodes, current_node_index + 1, labels, node_index)?;

                let node = node_index.get_mut(&unresolved_if_node.address).unwrap();
                if let LogicASTNode::If(if_node) = node {
                    if_node.then = Some(next_node_id);
                }
            }

            // insert a virtual goto at the end of the code for the skip target
            let goto_node = UnresolvedLogicASTNode::Goto(UnresolvedGotoNode {
                address: goto_node_address,
                jump_target_address: unresolved_if_node.else_goto_address,
            });
            unresolved_nodes.push(goto_node);
            let goto_node_id =
                resolve_nodes(unresolved_nodes, goto_node_index, labels, node_index)?;
            let node = node_index.get_mut(&unresolved_if_node.address).unwrap();
            if let LogicASTNode::If(if_node) = node {
                if_node.else_ = Some(goto_node_id.clone());
            }

            Ok(node_id)
        }
        UnresolvedLogicASTNode::Goto(unresolved_goto_node) => {
            let target = node_index.get(&unresolved_goto_node.jump_target_address);

            let target_id = match target {
                Some(target) => target.id().clone(),
                None => {
                    let target_index = unresolved_nodes
                        .iter()
                        .position(|n| n.address() == unresolved_goto_node.jump_target_address)
                        .ok_or_else(|| DecompilationError::InvalidJump {
                            target_address: unresolved_goto_node.jump_target_address,
                            current_address: unresolved_goto_node.address,
                        })?;

                    resolve_nodes(unresolved_nodes, target_index, labels, node_index)?
                }
            };

            let goto_node = LogicASTNode::Goto(LogicGotoNode {
                id: node_id.clone(),
                jump_target: target_id,
                label: labels.get(&unresolved_goto_node.address).cloned(),
                metadata: LogicASTNodeMetadata {
                    instruction_address: Some(unresolved_goto_node.address),
                },
            });
            node_index.insert(unresolved_goto_node.address, goto_node);
            Ok(node_id)
        }
    }
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use crate::{
        agi_version::AGIVersion,
        logic::{logic_script::ast::LogicAST, LogicProgram},
        resources::{
            decode::Decode,
            dirs::{ResourceDirDecodeOptions, ResourceDirs},
            resource_collection::{ResourceCollection, ResourceCollectionVersionData},
            ResourceType,
        },
        TEST_DATA_DIR,
    };

    #[test]
    fn test_build_ast() {
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

        let ast = LogicAST::from_instructions(&logic_program.instructions)
            .expect("Failed to build AST from instructions");
        assert!(
            !ast.root_node().id().is_empty(),
            "AST should have a root node"
        );
    }
}

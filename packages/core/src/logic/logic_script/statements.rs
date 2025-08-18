use strum_macros::AsRefStr;

#[cfg(feature = "dot")]
use crate::logic::logic_script::codegen::context::LogicScriptCodeGenerationContext;
use crate::logic::{
    asm::expressions::{
        AsParsedLogicArgument, LogicArgument, LogicBooleanExpression, LogicIdentifier,
        ParsedLogicArgument,
    },
    logic_script::{
        codegen::node_label_map::LabeledNode,
        directives::LogicScriptDirective,
        operators::{LogicScriptArithmeticOperator, LogicScriptUnaryAssignmentOperator},
    },
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LogicScriptCommandCall<Arg: LogicArgument> {
    pub command_name: String,
    pub argument_list: Vec<Arg>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LogicScriptIfStatement<Arg: LogicArgument, StatementType> {
    pub conditions: LogicBooleanExpression<Arg>,
    pub then_statements: Vec<StatementType>,
    pub else_statements: Vec<StatementType>,
    pub if_keyword: LogicScriptKeyword,
    pub else_keyword: Option<LogicScriptKeyword>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LogicScriptUnaryOperationStatement {
    pub operation: LogicScriptUnaryAssignmentOperator,
    pub identifier: LogicIdentifier,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LogicScriptValueAssignmentStatement<Arg: LogicArgument> {
    pub assignee: LogicIdentifier,
    pub value: Arg,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LogicScriptArithmeticAssignmentStatement<Arg: LogicArgument> {
    pub operator: LogicScriptArithmeticOperator,
    pub assignee: LogicIdentifier,
    pub value: Arg,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LogicScriptLeftIndirectAssignmentStatement<Arg: LogicArgument> {
    pub assignee_pointer: LogicIdentifier,
    pub value: Arg,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LogicScriptRightIndirectAssignmentStatement {
    pub assignee: LogicIdentifier,
    pub value_pointer: LogicIdentifier,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct LogicScriptComment {
    pub comment: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum KeywordType {
    If,
    Else,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LogicScriptKeyword {
    pub keyword: KeywordType,
}

#[derive(Debug, Clone, PartialEq, Eq, AsRefStr)]
pub enum LogicScriptStatementBody<Arg: LogicArgument> {
    CommandCall(LogicScriptCommandCall<Arg>),
    IfStatement(LogicScriptIfStatement<Arg, Box<LogicScriptStatement<Arg>>>),
    Comment(LogicScriptComment),
    UnaryOperation(LogicScriptUnaryOperationStatement),
    Directive(LogicScriptDirective),
    ValueAssignment(LogicScriptValueAssignmentStatement<Arg>),
    ArithmeticAssignment(LogicScriptArithmeticAssignmentStatement<Arg>),
    LeftIndirectAssignment(LogicScriptLeftIndirectAssignmentStatement<Arg>),
    RightIndirectAssignment(LogicScriptRightIndirectAssignmentStatement),
}

impl<Arg: LogicArgument + AsParsedLogicArgument> LogicScriptStatementBody<Arg> {
    pub fn to_parsed(&self) -> LogicScriptStatementBody<ParsedLogicArgument> {
        match self {
            LogicScriptStatementBody::CommandCall(body) => {
                LogicScriptStatementBody::CommandCall(LogicScriptCommandCall {
                    command_name: body.command_name.clone(),
                    argument_list: body
                        .argument_list
                        .iter()
                        .map(AsParsedLogicArgument::as_parsed)
                        .cloned()
                        .collect(),
                })
            }
            LogicScriptStatementBody::IfStatement(body) => {
                LogicScriptStatementBody::IfStatement(LogicScriptIfStatement {
                    conditions: body.conditions.to_parsed(),
                    then_statements: body
                        .then_statements
                        .iter()
                        .map(|s| Box::new(s.to_parsed()))
                        .collect(),
                    else_statements: body
                        .else_statements
                        .iter()
                        .map(|s| Box::new(s.to_parsed()))
                        .collect(),
                    if_keyword: body.if_keyword.clone(),
                    else_keyword: body.else_keyword.clone(),
                })
            }
            LogicScriptStatementBody::ValueAssignment(body) => {
                LogicScriptStatementBody::ValueAssignment(LogicScriptValueAssignmentStatement {
                    assignee: body.assignee.clone(),
                    value: body.value.as_parsed().clone(),
                })
            }
            LogicScriptStatementBody::ArithmeticAssignment(body) => {
                LogicScriptStatementBody::ArithmeticAssignment(
                    LogicScriptArithmeticAssignmentStatement {
                        operator: body.operator.clone(),
                        assignee: body.assignee.clone(),
                        value: body.value.as_parsed().clone(),
                    },
                )
            }
            LogicScriptStatementBody::LeftIndirectAssignment(body) => {
                LogicScriptStatementBody::LeftIndirectAssignment(
                    LogicScriptLeftIndirectAssignmentStatement {
                        assignee_pointer: body.assignee_pointer.clone(),
                        value: body.value.as_parsed().clone(),
                    },
                )
            }
            LogicScriptStatementBody::Comment(body) => {
                LogicScriptStatementBody::Comment(body.clone())
            }
            LogicScriptStatementBody::UnaryOperation(body) => {
                LogicScriptStatementBody::UnaryOperation(body.clone())
            }
            LogicScriptStatementBody::Directive(body) => {
                LogicScriptStatementBody::Directive(body.clone())
            }
            LogicScriptStatementBody::RightIndirectAssignment(body) => {
                LogicScriptStatementBody::RightIndirectAssignment(body.clone())
            }
        }
    }

    pub fn get_goto_target_label(&self) -> Option<&String> {
        let LogicScriptStatementBody::CommandCall(statement) = self else {
            return None;
        };

        if statement.command_name != "goto" {
            return None;
        }

        let Some(ParsedLogicArgument::Identifier(target)) = statement
            .argument_list
            .first()
            .map(AsParsedLogicArgument::as_parsed)
        else {
            return None;
        };

        Some(&target.name)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LogicScriptStatement<Arg: LogicArgument> {
    pub body: LogicScriptStatementBody<Arg>,
    pub label: Option<String>,
}

impl<Arg: LogicArgument> LogicScriptStatement<Arg> {
    pub fn new(body: LogicScriptStatementBody<Arg>, label: Option<String>) -> Self {
        Self { body, label }
    }
}

impl<Arg: LogicArgument + AsParsedLogicArgument> LogicScriptStatement<Arg> {
    pub fn to_parsed(&self) -> LogicScriptStatement<ParsedLogicArgument> {
        LogicScriptStatement {
            body: self.body.to_parsed(),
            label: self.label.clone(),
        }
    }
}

impl<Arg: LogicArgument> LabeledNode for LogicScriptStatement<Arg> {
    fn label(&self) -> Option<&str> {
        self.label.as_deref()
    }

    fn set_label(&mut self, label: Option<&str>) {
        self.label = label.map(|s| s.to_string());
    }
}

impl<Arg: LogicArgument + AsParsedLogicArgument> LogicScriptStatement<Arg> {
    pub fn get_goto_target_label(&self) -> Option<&String> {
        self.body.get_goto_target_label()
    }
}

#[cfg(feature = "dot")]
impl<Arg: LogicArgument + AsParsedLogicArgument + Clone> LogicScriptStatement<Arg> {
    pub fn dot_node_label(&self, context: &LogicScriptCodeGenerationContext) -> String {
        use crate::logic::logic_script::codegen::codegen::GenerateLogicScript;

        let label = match &self.body {
            LogicScriptStatementBody::IfStatement(if_statement) => {
                let header = format!(
                    "if ({})",
                    if_statement
                        .conditions
                        .generate_logic_script(&context, ())
                        .unwrap()
                );

                match &self.label {
                    Some(label) => format!("{}: {}", label, header),
                    None => header,
                }
            }
            _ => self
                .generate_logic_script(&context, 0)
                .unwrap()
                .trim()
                .to_string(),
        };

        if label.len() > 50 {
            format!("{}...", &label[0..50])
        } else {
            label
        }
    }

    pub fn dot_node_shape(&self) -> &str {
        match &self.body {
            LogicScriptStatementBody::Directive(_) => "oval",
            LogicScriptStatementBody::IfStatement(_) => "diamond",
            LogicScriptStatementBody::Comment(_) => "parallelogram",
            LogicScriptStatementBody::CommandCall(command_call) => {
                if command_call.command_name == "goto" {
                    "invtriangle"
                } else {
                    "box"
                }
            }
            _ => "box",
        }
    }

    pub fn dot_node_attrs(&self, context: &LogicScriptCodeGenerationContext) -> String {
        let label = self.dot_node_label(context);
        let shape = self.dot_node_shape();

        format!(
            "shape = {shape}, label = {}",
            serde_json::to_string(&label).expect("Failed to generate JSON")
        )
    }
}

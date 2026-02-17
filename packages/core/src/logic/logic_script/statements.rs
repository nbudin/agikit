use std::fmt::Debug;

use strum_macros::AsRefStr;

#[cfg(feature = "dot")]
use crate::logic::asm::codegen::AsmCodeGenerationContext;
use crate::logic::{
    asm::expressions::{
        AsParsedLogicArgument, LogicArgument, LogicBooleanExpression, LogicIdentifier,
        ParsedLogicArgument,
    },
    logic_script::{
        codegen::node_label_map::LabeledNode,
        directives::LogicScriptDirective,
        locations::{Locatable, ScriptLocationRange, WithLocation},
        operators::{LogicScriptArithmeticOperator, LogicScriptUnaryAssignmentOperator},
    },
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LogicScriptCommandCall<Arg: LogicArgument> {
    pub command_name: String,
    pub argument_list: Vec<Arg>,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct LogicScriptIfStatement<Arg: LogicArgument> {
    pub conditions: LogicBooleanExpression<Arg>,
    pub then_statements: Vec<StatementWithOrWithoutLocation<Arg>>,
    pub else_statements: Vec<StatementWithOrWithoutLocation<Arg>>,
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

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum StatementWithOrWithoutLocation<Arg: LogicArgument> {
    WithLocation(WithLocation<LogicScriptStatement<Arg>>),
    WithoutLocation(LogicScriptStatement<Arg>),
}

impl<Arg: LogicArgument> AsRef<LogicScriptStatement<Arg>> for StatementWithOrWithoutLocation<Arg> {
    fn as_ref(&self) -> &LogicScriptStatement<Arg> {
        self.statement()
    }
}

impl<Arg: LogicArgument> StatementWithOrWithoutLocation<Arg> {
    pub fn to_parsed(&self) -> StatementWithOrWithoutLocation<ParsedLogicArgument>
    where
        Arg: AsParsedLogicArgument,
    {
        match self {
            StatementWithOrWithoutLocation::WithLocation(stmt_with_location) => {
                StatementWithOrWithoutLocation::WithLocation(
                    stmt_with_location
                        .value
                        .to_parsed()
                        .with_location(stmt_with_location.location.clone()),
                )
            }
            StatementWithOrWithoutLocation::WithoutLocation(stmt) => {
                StatementWithOrWithoutLocation::WithoutLocation(stmt.to_parsed())
            }
        }
    }

    pub fn statement(&self) -> &LogicScriptStatement<Arg> {
        match self {
            StatementWithOrWithoutLocation::WithLocation(stmt_with_location) => {
                &stmt_with_location.value
            }
            StatementWithOrWithoutLocation::WithoutLocation(stmt) => stmt,
        }
    }

    pub fn location(&self) -> Option<&ScriptLocationRange> {
        match self {
            StatementWithOrWithoutLocation::WithLocation(stmt_with_location) => {
                Some(&stmt_with_location.location)
            }
            StatementWithOrWithoutLocation::WithoutLocation(_) => None,
        }
    }

    pub fn with_default_location(&self) -> WithLocation<LogicScriptStatement<Arg>>
    where
        Arg: Clone,
    {
        match self {
            StatementWithOrWithoutLocation::WithLocation(with_location) => with_location.clone(),
            StatementWithOrWithoutLocation::WithoutLocation(logic_script_statement) => {
                logic_script_statement
                    .clone()
                    .with_location(ScriptLocationRange::default())
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, AsRefStr)]
pub enum LogicScriptStatementBody<Arg: LogicArgument> {
    CommandCall(LogicScriptCommandCall<Arg>),
    IfStatement(LogicScriptIfStatement<Arg>),
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
                    then_statements: body.then_statements.iter().map(|s| s.to_parsed()).collect(),
                    else_statements: body.else_statements.iter().map(|s| s.to_parsed()).collect(),
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

impl<Arg: LogicArgument> LabeledNode for LogicScriptStatement<Arg> {
    fn label(&self) -> Option<&str> {
        self.label.as_deref()
    }

    fn set_label(&mut self, label: Option<&str>) {
        self.label = label.map(|s| s.to_string());
    }
}

impl<Arg: LogicArgument + AsParsedLogicArgument> LogicScriptStatement<Arg> {
    pub fn to_parsed(&self) -> LogicScriptStatement<ParsedLogicArgument> {
        LogicScriptStatement {
            body: self.body.to_parsed(),
            label: self.label.clone(),
        }
    }

    pub fn get_goto_target_label(&self) -> Option<&String> {
        self.body.get_goto_target_label()
    }
}

#[cfg(feature = "dot")]
impl<Arg: LogicArgument + AsParsedLogicArgument + Clone> LogicScriptStatement<Arg> {
    pub fn dot_node_label(&self, context: &AsmCodeGenerationContext) -> String {
        use crate::logic::logic_script::codegen::codegen::GenerateLogicScript;

        let label = match &self.body {
            LogicScriptStatementBody::IfStatement(if_statement) => {
                let header = format!(
                    "if ({})",
                    if_statement
                        .conditions
                        .generate_logic_script(context, ())
                        .unwrap()
                );

                match &self.label {
                    Some(label) => format!("{}: {}", label, header),
                    None => header,
                }
            }
            _ => self
                .generate_logic_script(context, 0)
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

    pub fn dot_node_attrs(&self, context: &AsmCodeGenerationContext) -> String {
        let label = self.dot_node_label(context);
        let shape = self.dot_node_shape();

        format!(
            "shape = {shape}, label = {}",
            serde_json::to_string(&label).expect("Failed to generate JSON")
        )
    }
}

use crate::logic::logic_script::{
    directives::LogicScriptDirective,
    expressions::{
        LogicScriptArgument, LogicScriptArgumentList, LogicScriptBooleanExpression,
        LogicScriptIdentifier,
    },
    operators::{LogicScriptArithmeticOperator, LogicScriptUnaryAssignmentOperator},
    parsing::ScriptLocationRange,
};

#[derive(Debug)]
pub struct LogicScriptCommandCall {
    pub commmand_name: String,
    pub argument_list: LogicScriptArgumentList,
    pub location: Option<ScriptLocationRange>,
    pub command_name_location: Option<ScriptLocationRange>,
}

#[derive(Debug)]
pub struct LogicScriptIfStatement<StatementType> {
    pub conditions: LogicScriptBooleanExpression,
    pub then_statements: Vec<StatementType>,
    pub else_statements: Vec<StatementType>,
    pub if_keyword: LogicScriptKeyword,
    pub else_keyword: Option<LogicScriptKeyword>,
    pub location: Option<ScriptLocationRange>,
}

#[derive(Debug)]
pub struct LogicScriptUnaryOperationStatement {
    pub operation: LogicScriptUnaryAssignmentOperator,
    pub identifier: LogicScriptIdentifier,
    pub location: Option<ScriptLocationRange>,
}

#[derive(Debug)]
pub struct LogicScriptValueAssignmentStatement {
    pub assignee: LogicScriptIdentifier,
    pub value: LogicScriptArgument,
    pub location: Option<ScriptLocationRange>,
}

#[derive(Debug)]
pub struct LogicScriptArithmeticAssignmentStatement {
    pub operator: LogicScriptArithmeticOperator,
    pub assignee: LogicScriptIdentifier,
    pub value: LogicScriptArgument,
    pub location: Option<ScriptLocationRange>,
}

#[derive(Debug)]
pub struct LogicScriptLeftIndirectAssignmentStatement {
    pub assignee_pointer: LogicScriptIdentifier,
    pub value: LogicScriptArgument,
    pub location: Option<ScriptLocationRange>,
}

#[derive(Debug)]
pub struct LogicScriptRightIndirectAssignmentStatement {
    pub assignee: LogicScriptIdentifier,
    pub value_pointer: LogicScriptIdentifier,
    pub location: Option<ScriptLocationRange>,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct LogicScriptComment {
    pub comment: String,
    pub location: Option<ScriptLocationRange>,
}

#[derive(Debug, Clone)]
pub enum KeywordType {
    If,
    Else,
}

#[derive(Debug, Clone)]
pub struct LogicScriptKeyword {
    pub keyword: KeywordType,
    pub location: Option<ScriptLocationRange>,
}

#[derive(Debug)]
pub struct LogicScriptLabel {
    pub label: String,
    pub location: Option<ScriptLocationRange>,
}

#[derive(Debug)]
pub enum LogicScriptStatement {
    Label(LogicScriptLabel),
    CommandCall(LogicScriptCommandCall),
    IfStatement(LogicScriptIfStatement<Box<LogicScriptStatement>>),
    Comment(LogicScriptComment),
    UnaryOperation(LogicScriptUnaryOperationStatement),
    Directive(LogicScriptDirective),
    ValueAssignment(LogicScriptValueAssignmentStatement),
    ArithmeticAssignment(LogicScriptArithmeticAssignmentStatement),
    LeftIndirectAssignment(LogicScriptLeftIndirectAssignmentStatement),
    RightIndirectAssignment(LogicScriptRightIndirectAssignmentStatement),
}

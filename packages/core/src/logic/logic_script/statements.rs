use crate::logic::{
    asm::expressions::{LogicArgument, LogicBooleanExpression, LogicIdentifier},
    logic_script::{
        directives::LogicScriptDirective,
        operators::{LogicScriptArithmeticOperator, LogicScriptUnaryAssignmentOperator},
    },
};

#[derive(Debug)]
pub struct LogicScriptCommandCall<Arg: LogicArgument> {
    pub commmand_name: String,
    pub argument_list: Vec<Arg>,
}

#[derive(Debug)]
pub struct LogicScriptIfStatement<Arg: LogicArgument, StatementType> {
    pub conditions: LogicBooleanExpression<Arg>,
    pub then_statements: Vec<StatementType>,
    pub else_statements: Vec<StatementType>,
    pub if_keyword: LogicScriptKeyword,
    pub else_keyword: Option<LogicScriptKeyword>,
}

#[derive(Debug)]
pub struct LogicScriptUnaryOperationStatement {
    pub operation: LogicScriptUnaryAssignmentOperator,
    pub identifier: LogicIdentifier,
}

#[derive(Debug)]
pub struct LogicScriptValueAssignmentStatement<Arg: LogicArgument> {
    pub assignee: LogicIdentifier,
    pub value: Arg,
}

#[derive(Debug)]
pub struct LogicScriptArithmeticAssignmentStatement<Arg: LogicArgument> {
    pub operator: LogicScriptArithmeticOperator,
    pub assignee: LogicIdentifier,
    pub value: Arg,
}

#[derive(Debug)]
pub struct LogicScriptLeftIndirectAssignmentStatement<Arg: LogicArgument> {
    pub assignee_pointer: LogicIdentifier,
    pub value: Arg,
}

#[derive(Debug)]
pub struct LogicScriptRightIndirectAssignmentStatement {
    pub assignee: LogicIdentifier,
    pub value_pointer: LogicIdentifier,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct LogicScriptComment {
    pub comment: String,
}

#[derive(Debug, Clone)]
pub enum KeywordType {
    If,
    Else,
}

#[derive(Debug, Clone)]
pub struct LogicScriptKeyword {
    pub keyword: KeywordType,
}

#[derive(Debug)]
pub struct LogicScriptLabel {
    pub label: String,
}

#[derive(Debug)]
pub enum LogicScriptStatement<Arg: LogicArgument> {
    Label(LogicScriptLabel),
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

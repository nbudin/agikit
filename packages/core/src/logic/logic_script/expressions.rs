use crate::logic::logic_script::{
    literals::LogicScriptLiteral, operators::LogicScriptBooleanBinaryOperator,
    parsing::ScriptLocationRange,
};

#[derive(Debug)]
pub struct LogicScriptIdentifier {
    pub name: String,
    pub location: Option<ScriptLocationRange>,
}

#[derive(Debug)]
pub enum LogicScriptArgument {
    Literal(LogicScriptLiteral),
    Identifier(LogicScriptIdentifier),
}

pub type LogicScriptArgumentList = Vec<LogicScriptArgument>;

#[derive(Debug)]
pub struct LogicScriptAndExpression {
    pub clauses: Vec<LogicScriptBooleanExpression>,
    pub location: Option<ScriptLocationRange>,
}

#[derive(Debug)]
pub struct LogicScriptOrExpression {
    pub clauses: Vec<LogicScriptBooleanExpression>,
    pub location: Option<ScriptLocationRange>,
}

#[derive(Debug)]
pub struct LogicScriptNotExpression {
    pub expression: Box<LogicScriptBooleanExpression>,
    pub location: Option<ScriptLocationRange>,
}

#[derive(Debug)]
pub struct LogicScriptBooleanBinaryOperation {
    pub left: LogicScriptArgument,
    pub operator: LogicScriptBooleanBinaryOperator,
    pub right: LogicScriptArgument,
    pub location: Option<ScriptLocationRange>,
}

#[derive(Debug)]
pub struct LogicScriptTestCall {
    pub test_name: String,
    pub argument_list: LogicScriptArgumentList,
    pub location: Option<ScriptLocationRange>,
    pub test_name_location: Option<ScriptLocationRange>,
}

#[derive(Debug)]
pub enum LogicScriptBooleanExpression {
    BinaryOperation(LogicScriptBooleanBinaryOperation),
    AndExpression(LogicScriptAndExpression),
    OrExpression(LogicScriptOrExpression),
    NotExpression(LogicScriptNotExpression),
    TestCall(LogicScriptTestCall),
    Identifier(LogicScriptIdentifier),
}

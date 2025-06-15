use crate::logic::logic_script::{
    expressions::LogicScriptIdentifier,
    literals::{LogicScriptLiteral, LogicScriptNumberLiteral, LogicScriptStringLiteral},
    parsing::ScriptLocationRange,
};

#[derive(Debug)]
pub enum LogicScriptDefineValue {
    Literal(LogicScriptLiteral),
    Identifier(LogicScriptIdentifier),
}

#[derive(Debug)]
pub enum Directive {
    Message {
        number: LogicScriptNumberLiteral,
        message: LogicScriptStringLiteral,
    },
    Include {
        filename: LogicScriptStringLiteral,
    },
    Define {
        identifier: LogicScriptIdentifier,
        value: LogicScriptDefineValue,
    },
}

#[derive(Debug)]
pub enum DirectiveType {
    Message,
    Include,
    Define,
}

#[derive(Debug)]
pub struct LogicScriptDirectiveKeyword {
    pub keyword: DirectiveType,
    pub location: Option<ScriptLocationRange>,
}

#[derive(Debug)]
pub struct LogicScriptDirective {
    pub directive: Directive,
    pub keyword: LogicScriptDirectiveKeyword,
    pub location: Option<ScriptLocationRange>,
}

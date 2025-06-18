use crate::logic::{
    asm::{expressions::LogicIdentifier, literals::LogicNumberLiteral},
    logic_script::literals::{LogicScriptLiteral, LogicScriptStringLiteral},
};

#[derive(Debug)]
pub enum LogicScriptDefineValue {
    Literal(LogicScriptLiteral),
    Identifier(LogicIdentifier),
}

#[derive(Debug)]
pub enum Directive {
    Message {
        number: LogicNumberLiteral,
        message: LogicScriptStringLiteral,
    },
    Include {
        filename: LogicScriptStringLiteral,
    },
    Define {
        identifier: LogicIdentifier,
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
}

#[derive(Debug)]
pub struct LogicScriptDirective {
    pub directive: Directive,
    pub keyword: LogicScriptDirectiveKeyword,
}

use crate::logic::{
    asm::{expressions::LogicIdentifier, literals::LogicNumberLiteral},
    logic_script::literals::{LogicScriptLiteral, LogicScriptStringLiteral},
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LogicScriptDefineValue {
    Literal(LogicScriptLiteral),
    Identifier(LogicIdentifier),
}

#[derive(Debug, Clone, PartialEq, Eq)]
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DirectiveType {
    Message,
    Include,
    Define,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LogicScriptDirectiveKeyword {
    pub keyword: DirectiveType,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LogicScriptDirective {
    pub directive: Directive,
    pub keyword: LogicScriptDirectiveKeyword,
}

use crate::logic::{
    asm::literals::{
        LogicLiteral, LogicLiteralValue, LogicNumberLiteral, LogicStringLiteral, StringLiteral,
    },
    logic_script::statements::LogicScriptComment,
};

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct LogicScriptSingleStringLiteral {
    pub value: String,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum LogicScriptStringLiteralPart {
    SingleString(LogicScriptSingleStringLiteral),
    Comment(LogicScriptComment),
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct LogicScriptStringLiteral {
    pub parts: Vec<LogicScriptStringLiteralPart>,
}

impl LogicScriptStringLiteral {
    pub fn value(&self) -> String {
        self.parts
            .iter()
            .filter_map(|part| match part {
                LogicScriptStringLiteralPart::SingleString(single) => Some(single.value.as_str()),
                LogicScriptStringLiteralPart::Comment(_) => None,
            })
            .collect()
    }
}

impl StringLiteral<'_, String> for LogicScriptStringLiteral {
    fn value(&self) -> String {
        self.value()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LogicScriptLiteralValue {
    Number(LogicNumberLiteral),
    String(LogicScriptStringLiteral),
}

impl LogicScriptLiteralValue {
    pub fn from_string(value: String) -> Self {
        LogicScriptLiteralValue::String(LogicScriptStringLiteral {
            parts: vec![LogicScriptStringLiteralPart::SingleString(
                LogicScriptSingleStringLiteral { value },
            )],
        })
    }
}

impl From<LogicScriptLiteral> for LogicLiteral {
    fn from(literal: LogicScriptLiteral) -> Self {
        let value = match literal.value {
            LogicScriptLiteralValue::Number(num) => {
                LogicLiteralValue::Number(LogicNumberLiteral { value: num.value })
            }
            LogicScriptLiteralValue::String(string) => {
                LogicLiteralValue::String(LogicStringLiteral {
                    value: string.value(),
                })
            }
        };
        LogicLiteral { value }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LogicScriptLiteral {
    pub value: LogicScriptLiteralValue,
}

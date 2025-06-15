use crate::logic::logic_script::{parsing::ScriptLocationRange, statements::LogicScriptComment};

#[derive(Debug)]
pub struct LogicScriptNumberLiteral {
    pub value: i32,
    pub location: Option<ScriptLocationRange>,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct LogicScriptSingleStringLiteral {
    pub value: String,
    pub location: Option<ScriptLocationRange>,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum LogicScriptStringLiteralPart {
    SingleString(LogicScriptSingleStringLiteral),
    Comment(LogicScriptComment),
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct LogicScriptStringLiteral {
    pub parts: Vec<LogicScriptStringLiteralPart>,
    pub location: Option<ScriptLocationRange>,
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

#[derive(Debug)]
pub enum LogicScriptLiteralValue {
    Number(LogicScriptNumberLiteral),
    String(LogicScriptStringLiteral),
}

#[derive(Debug)]
pub struct LogicScriptLiteral {
    pub value: LogicScriptLiteralValue,
    pub location: Option<ScriptLocationRange>,
}

#[derive(Debug, Clone)]
pub struct LogicNumberLiteral {
    pub value: i32,
}

pub trait StringLiteral<'a, Output: ToString + 'a> {
    fn value(&'a self) -> Output;
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct LogicStringLiteral {
    pub value: String,
}

impl<'a> StringLiteral<'a, &'a str> for LogicStringLiteral {
    fn value(&'a self) -> &'a str {
        &self.value
    }
}

#[derive(Debug, Clone)]
pub enum LogicLiteralValue {
    Number(LogicNumberLiteral),
    String(LogicStringLiteral),
}

impl LogicLiteralValue {
    pub fn from_string(value: String) -> Self {
        LogicLiteralValue::String(LogicStringLiteral { value })
    }
}

#[derive(Debug, Clone)]
pub struct LogicLiteral {
    pub value: LogicLiteralValue,
}

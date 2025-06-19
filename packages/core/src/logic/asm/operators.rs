use serde::{Deserialize, Serialize};

pub trait LogicBinaryOperator {
    fn is_commutative(&self) -> bool;
}

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum LogicBooleanBinaryOperator {
    #[serde(rename = "<")]
    LessThan,
    #[serde(rename = "<=")]
    LessThanOrEqual,
    #[serde(rename = ">")]
    GreaterThan,
    #[serde(rename = ">=")]
    GreaterThanOrEqual,
    #[serde(rename = "==")]
    Equal,
    #[serde(rename = "!=")]
    NotEqual,
}

impl LogicBinaryOperator for LogicBooleanBinaryOperator {
    fn is_commutative(&self) -> bool {
        matches!(
            self,
            LogicBooleanBinaryOperator::Equal | LogicBooleanBinaryOperator::NotEqual
        )
    }
}

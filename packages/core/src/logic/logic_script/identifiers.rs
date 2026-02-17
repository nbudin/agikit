use std::{
    collections::{HashMap, hash_map::Entry},
    fmt::Display,
    num::TryFromIntError,
};

use crate::logic::{
    commands::AGICommandArgType,
    logic_script::{directives::LogicScriptDefineValue, literals::LogicScriptLiteralValue},
};

#[derive(Debug, Clone)]
pub enum IdentifierMapping {
    Variable {
        name: String,
        number: u16,
        variable_type: AGICommandArgType,
    },
    ConstantString {
        name: String,
        value: String,
    },
    ConstantNumber {
        name: String,
        value: u8,
    },
}

impl IdentifierMapping {
    pub fn builtins() -> impl Iterator<Item = IdentifierMapping> {
        (0..256).flat_map(|index| {
            [
                IdentifierMapping::Variable {
                    name: format!("v{}", index),
                    number: index,
                    variable_type: AGICommandArgType::Variable,
                },
                IdentifierMapping::Variable {
                    name: format!("f{}", index),
                    number: index,
                    variable_type: AGICommandArgType::Flag,
                },
                IdentifierMapping::Variable {
                    name: format!("o{}", index),
                    number: index,
                    variable_type: AGICommandArgType::Object,
                },
                IdentifierMapping::Variable {
                    name: format!("c{}", index),
                    number: index,
                    variable_type: AGICommandArgType::CtrlCode,
                },
                IdentifierMapping::Variable {
                    name: format!("i{}", index),
                    number: index,
                    variable_type: AGICommandArgType::Item,
                },
                IdentifierMapping::Variable {
                    name: format!("s{}", index),
                    number: index,
                    variable_type: AGICommandArgType::String,
                },
                IdentifierMapping::Variable {
                    name: format!("m{}", index),
                    number: index,
                    variable_type: AGICommandArgType::Message,
                },
                IdentifierMapping::Variable {
                    name: format!("w{}", index),
                    number: index,
                    variable_type: AGICommandArgType::Word,
                },
            ]
        })
    }

    pub fn name(&self) -> &str {
        match self {
            IdentifierMapping::Variable { name, .. } => name,
            IdentifierMapping::ConstantString { name, .. } => name,
            IdentifierMapping::ConstantNumber { name, .. } => name,
        }
    }
}

#[derive(Debug, Clone)]
pub enum DefineError {
    TryFromIntError(TryFromIntError),
    UnknownIdentifier { name: String },
    IdentifierAlreadyDefined { name: String },
}

impl Display for DefineError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DefineError::TryFromIntError(try_from_int_error) => try_from_int_error.fmt(f),
            DefineError::UnknownIdentifier { name } => {
                f.write_fmt(format_args!("Unknown identifier: {}", name))
            }
            DefineError::IdentifierAlreadyDefined { name } => {
                f.write_fmt(format_args!("Identifier {} already defined", name))
            }
        }
    }
}

impl From<TryFromIntError> for DefineError {
    fn from(value: TryFromIntError) -> Self {
        Self::TryFromIntError(value)
    }
}

#[derive(Debug, Clone)]
pub struct IdentifierMap(HashMap<String, IdentifierMapping>);

impl IdentifierMap {
    pub fn builtins() -> Self {
        Self::from_identifiers(IdentifierMapping::builtins())
    }

    pub fn from_identifiers(identifiers: impl IntoIterator<Item = IdentifierMapping>) -> Self {
        Self(
            identifiers
                .into_iter()
                .map(|identifier| (identifier.name().to_string(), identifier))
                .collect(),
        )
    }

    pub fn get(&self, name: &str) -> Option<&IdentifierMapping> {
        self.0.get(name)
    }

    pub fn define(
        &mut self,
        name: String,
        value: &LogicScriptDefineValue,
    ) -> Result<&IdentifierMapping, DefineError> {
        let mapping = match value {
            LogicScriptDefineValue::Literal(literal) => match &literal.value {
                LogicScriptLiteralValue::Number(number) => IdentifierMapping::ConstantNumber {
                    name: name.clone(),
                    value: number.value.try_into()?,
                },
                LogicScriptLiteralValue::String(string) => IdentifierMapping::ConstantString {
                    name: name.clone(),
                    value: string.value(),
                },
            },
            LogicScriptDefineValue::Identifier(identifier) => {
                let referent = self.0.get(&identifier.name);
                match referent {
                    Some(referent) => referent.clone(),
                    None => {
                        return Err(DefineError::UnknownIdentifier {
                            name: identifier.name.clone(),
                        });
                    }
                }
            }
        };

        let entry = self.0.entry(name.clone());
        match entry {
            Entry::Occupied(_) => {
                return Err(DefineError::IdentifierAlreadyDefined { name: name.clone() });
            }
            Entry::Vacant(entry) => Ok(entry.insert(mapping)),
        }
    }
}

impl AsRef<HashMap<String, IdentifierMapping>> for IdentifierMap {
    fn as_ref(&self) -> &HashMap<String, IdentifierMapping> {
        &self.0
    }
}

impl AsMut<HashMap<String, IdentifierMapping>> for IdentifierMap {
    fn as_mut(&mut self) -> &mut HashMap<String, IdentifierMapping> {
        &mut self.0
    }
}

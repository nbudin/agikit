use std::collections::HashMap;

use crate::logic::commands::AGICommandArgType;

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
}

impl AsRef<HashMap<String, IdentifierMapping>> for IdentifierMap {
    fn as_ref(&self) -> &HashMap<String, IdentifierMapping> {
        &self.0
    }
}

use std::{collections::HashMap, sync::LazyLock};

use logic_command_macros::{include_agi_commands, include_test_commands};
use serde::{Deserialize, Serialize};
use strum_macros::{AsRefStr, EnumString};
use wasm_bindgen::prelude::wasm_bindgen;

use crate::agi_version::AGIVersion;

#[derive(Debug, Clone, Copy, PartialEq, Eq, EnumString, AsRefStr, Serialize, Deserialize)]
pub enum AGICommandArgType {
    Number,
    Variable,
    Flag,
    Message,
    Object,
    Item,
    String,
    Word,
    CtrlCode,
}

#[wasm_bindgen(skip_typescript)]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AGICommand {
    pub opcode: u8,
    #[wasm_bindgen(getter_with_clone)]
    pub name: String,
    #[wasm_bindgen(skip)]
    pub arg_types: Vec<AGICommandArgType>,
}

impl AGICommand {
    pub fn get(opcode: u8, agi_version: &AGIVersion) -> Option<&'static Self> {
        if opcode > 177 && agi_version < &AGIVersion::new(3, 2086) {
            return None;
        } else if opcode > 175 && agi_version < &AGIVersion::new(2, 936) {
            return None;
        } else if opcode > 173 && agi_version < &AGIVersion::new(2, 917) {
            return None;
        } else if opcode > 169 && agi_version < &AGIVersion::new(2, 440) {
            return None;
        } else if opcode > 161 && agi_version < &AGIVersion::new(2, 272) {
            return None;
        } else if opcode > 155 && agi_version < &AGIVersion::new(2, 89) {
            return None;
        }

        if opcode == 134 && agi_version == &AGIVersion::new(2, 89) {
            todo!("Opcode 134 with no args");
        }

        if (opcode == 151 || opcode == 152) && agi_version < &AGIVersion::new(2, 400) {
            todo!("print.at and print.at.v with only the first 2 args");
        }

        if opcode == 176 && agi_version < &AGIVersion::new(2, 2086) {
            todo!("hide.mouse with a single number arg");
        }

        AGI_COMMANDS.get(&opcode)
    }
}

#[wasm_bindgen(skip_typescript)]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TestCommand {
    pub opcode: u8,
    #[wasm_bindgen(getter_with_clone)]
    pub name: String,
    #[wasm_bindgen(skip)]
    pub arg_types: Vec<AGICommandArgType>,
    #[wasm_bindgen(js_name = "varArgs")]
    #[serde(default)]
    pub var_args: bool,
}

impl TestCommand {
    pub fn get(opcode: u8) -> Option<&'static Self> {
        TEST_COMMANDS.get(&opcode)
    }
}

pub static AGI_COMMANDS: LazyLock<HashMap<u8, AGICommand>> = LazyLock::new(|| {
    let commands: Vec<AGICommand> = include_agi_commands!("src/logic/agi_commands.json");
    commands.into_iter().map(|cmd| (cmd.opcode, cmd)).collect()
});

pub static TEST_COMMANDS: LazyLock<HashMap<u8, TestCommand>> = LazyLock::new(|| {
    let commands: Vec<TestCommand> = include_test_commands!("src/logic/test_commands.json");
    commands.into_iter().map(|cmd| (cmd.opcode, cmd)).collect()
});

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_agi_commands() {
        assert!(!AGI_COMMANDS.is_empty());
        assert_eq!(AGI_COMMANDS.get(&1).unwrap().name, "increment");
    }

    #[test]
    fn test_test_commands() {
        assert!(!TEST_COMMANDS.is_empty());
        assert_eq!(TEST_COMMANDS.get(&1).unwrap().name, "equaln");
    }
}

#[cfg(feature = "js")]
pub mod js {
    use std::collections::HashMap;

    use tsify::serde_wasm_bindgen;
    use wasm_bindgen::{prelude::wasm_bindgen, JsValue};

    use crate::{
        agi_version::AGIVersion,
        logic::commands::{AGICommand, TestCommand, AGI_COMMANDS, TEST_COMMANDS},
    };

    #[wasm_bindgen(typescript_custom_section)]
    const TS_APPEND_CONTENT: &'static str = r#"
export enum AGICommandArgType {
  Number = 'Number',
  Variable = 'Variable',
  Flag = 'Flag',
  Message = 'Message',
  Object = 'Object',
  Item = 'Item',
  String = 'String',
  Word = 'Word',
  CtrlCode = 'CtrlCode',
}

export class AGICommand {
  private constructor();
  free(): void;
  opcode: number;
  name: string;
  argTypes: AGICommandArgType[];
}

export class TestCommand {
  private constructor();
  free(): void;
  opcode: number;
  name: string;
  varArgs: boolean;
  argTypes: AGICommandArgType[];
}
"#;

    #[wasm_bindgen]
    impl AGICommand {
        #[wasm_bindgen(getter, js_name = "argTypes", skip_typescript)]
        pub fn js_args(&self) -> Vec<JsValue> {
            self.arg_types
                .iter()
                .map(|arg| JsValue::from_str(arg.as_ref()))
                .collect()
        }
    }

    #[wasm_bindgen(js_name = "getAGICommand")]
    pub fn js_get_agi_command(opcode: u8, agi_version: &AGIVersion) -> Option<AGICommand> {
        AGICommand::get(opcode, agi_version).cloned()
    }

    #[wasm_bindgen]
    impl TestCommand {
        #[wasm_bindgen(getter, js_name = "argTypes", skip_typescript)]
        pub fn js_args(&self) -> Vec<JsValue> {
            self.arg_types
                .iter()
                .map(|arg| JsValue::from_str(arg.as_ref()))
                .collect()
        }
    }

    #[wasm_bindgen(js_name = "getTestCommand")]
    pub fn js_get_test_command(opcode: u8) -> Option<TestCommand> {
        TestCommand::get(opcode).cloned()
    }

    #[wasm_bindgen(js_name = "getAGICommands")]
    pub fn get_agi_commands() -> Vec<AGICommand> {
        AGI_COMMANDS.values().cloned().collect()
    }

    #[wasm_bindgen(js_name = "getTestCommands")]
    pub fn get_test_commands() -> Vec<TestCommand> {
        TEST_COMMANDS.values().cloned().collect()
    }

    #[wasm_bindgen(js_name = "getAGICommandsByName")]
    pub fn get_agi_commands_by_name() -> Result<JsValue, serde_wasm_bindgen::Error> {
        let commands_by_name: HashMap<String, AGICommand> = AGI_COMMANDS
            .values()
            .map(|cmd| (cmd.name.clone(), cmd.clone()))
            .collect();

        serde_wasm_bindgen::to_value(&commands_by_name)
    }

    #[wasm_bindgen(js_name = "getTestCommandsByName")]
    pub fn get_test_commands_by_name() -> Result<JsValue, serde_wasm_bindgen::Error> {
        let commands_by_name: HashMap<String, TestCommand> = TEST_COMMANDS
            .values()
            .map(|cmd| (cmd.name.clone(), cmd.clone()))
            .collect();

        serde_wasm_bindgen::to_value(&commands_by_name)
    }
}

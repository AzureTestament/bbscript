use crate::error::BBScriptError;
use bimap::BiMap;
use ron::de;
use serde::Deserialize;

use std::error::Error;
use std::fs::File;
use std::path::PathBuf;

const DB_FOLDER: &str = "static_db";

#[derive(Deserialize, Debug)]
pub struct GameDB {
    functions: Vec<Function>,
}
impl GameDB {
    pub fn new(game: &str) -> Result<GameDB, Box<dyn Error>> {
        let mut cmd_db_path: PathBuf = PathBuf::from(DB_FOLDER);
        cmd_db_path.push(game);
        cmd_db_path.set_extension("ron");

        match File::open(&cmd_db_path) {
            Ok(file) => {
                let db: GameDB = de::from_reader(file)?;
                return Ok(db);
            },
            Err(_) => {
                let db_path_err = BBScriptError::GameDBNotFound(format!("{}", cmd_db_path.display()));
                return Err(Box::new(db_path_err));
            },
        }
    }

    pub fn find_by_id(&self, id_in: u32) -> Result<&Function, BBScriptError> {
        if let Some(func) = self.functions.iter().find(|x| x.id == id_in) {
            return Ok(func);
        } else {
            return Err(BBScriptError::UnknownFunction(format!("{:#X}", id_in)));
        }
    }

    pub fn find_by_name(&self, name_in: &str) -> Result<&Function, BBScriptError> {
        if let Some(func) = self.functions.iter().find(|x| x.name == name_in) {
            return Ok(func);
        } else {
            return Err(BBScriptError::UnknownFunction(name_in.into()));
        }
    }
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Function {
    pub id: u32,
    pub size: u32,
    args: String,
    pub name: String,
    pub code_block: CodeBlock,
    named_values: BiMap<(u32, i32), (u32, String)>,
}
impl Function {
    // Not recoverable because name has no inherent value
    pub fn get_value(&self, name: (u32, String)) -> Result<i32, BBScriptError> {
        if let Some(value) = self.named_values.get_by_right(&name) {
            return Ok(value.1);
        } else {
            Err(BBScriptError::NoAssociatedValue(name.0.to_string(), name.1))
        }
    }

    // Recoverable, will just use raw value if no name associated
    pub fn get_name(&self, value: (u32, i32)) -> Option<String> {
        if let Some(value) = self.named_values.get_by_left(&value) {
            return Some(value.1.clone());
        } else {
            return None;
        }
    }

    pub fn get_args(&self) -> Vec<Arg> {
        let arg_string = &self.args;

        let mut arg_accumulator = Vec::<Arg>::new();
        let mut arg_string = arg_string.as_bytes();
        let mut size_of_args = 0;

        while !arg_string.is_empty() {
            match arg_string {
                [b'i', ..] => {
                    size_of_args += 4;
                    arg_accumulator.push(Arg::Int);
                    arg_string = &arg_string[1..];
                }
                [b'1', b'6', b's', ..] => {
                    size_of_args += 16;
                    arg_accumulator.push(Arg::String16);
                    arg_string = &arg_string[3..];
                }
                [b'3', b'2', b's', ..] => {
                    size_of_args += 32;
                    arg_accumulator.push(Arg::String32);
                    arg_string = &arg_string[3..]
                }
                _ => arg_string = &arg_string[1..],
            }
        }
        if size_of_args < self.size - 4 {
            if self.size >= 4 {
                let left_over = self.size - size_of_args - 4;
                arg_accumulator.push(Arg::Unknown(left_over));
            }
        }
        return arg_accumulator;
    }

    pub fn instruction_name(&self) -> String {
        if self.name.is_empty() {
            return format!("Unknown{}", &self.id);
        } else {
            return self.name.to_string();
        }
    }

    pub fn is_jump_entry(&self) -> bool {
        self.code_block == CodeBlock::BeginJumpEntry
    }
}

#[derive(Debug)]
pub enum Arg {
    String16,
    String32,
    Int,
    Unknown(u32),
}

#[derive(Deserialize, Debug, PartialEq)]
pub enum CodeBlock {
    Begin,
    BeginJumpEntry,
    End,
    NoBlock,
}

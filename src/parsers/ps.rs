//A parser for `ps` command output

use anyhow::Error;
use serde_json::Value;

use super::universal::simple_table_parse;

pub struct PsParser {
    input: String,
}

impl PsParser {
    pub fn new(input: String) -> Self {
        PsParser {
            input: input,
        }
    }

    pub fn parse(&mut self) -> Result<Value, Error> {
        simple_table_parse(self.input.clone())
    }
}
//A parser for `ps` command output

use anyhow::Error;
use serde_json::Value;

use super::universal::sparse_table_parse;

pub struct NetstatParser {
    input: String,
}

impl NetstatParser {
    pub fn new(input: String) -> Self {
        NetstatParser {
            input: input,
        }
    }

    pub fn parse(&mut self) -> Result<Value, Error> {
        sparse_table_parse(self.input.clone())
    }
}
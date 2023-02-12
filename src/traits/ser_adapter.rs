use std::io::{BufReader, Read};

use polars::prelude::{DataFrame, JsonFormat, JsonWriter};
use polars_io::SerWriter;
use serde_json::{Error, Value};

use crate::converters::csv_parse;

use super::tabled::value_to_table;

#[derive(Debug)]
pub enum FormatOption {
    JSON,
    CSV,
    TSV,
    ARROW,
    TABLED,
}

pub trait ParseSerde {
    fn parse_serde(&mut self, format: &FormatOption) -> Result<Value, Error>;
}
pub trait IntoSerde {
    fn to_serde(&mut self) -> Result<Value, Error>;
}
pub trait FromSerde {
    fn to_table(&self) -> Result<String, ()>;
    fn to_json_string(&self) -> String;
    fn to_format_option(&self, format: &FormatOption) -> Result<String, ()>;
}

impl<T> ParseSerde for BufReader<T>
where
    T: Read,
{
    fn parse_serde(&mut self, format: &FormatOption) -> Result<Value, Error> {
        return match format {
            FormatOption::JSON => serde_json::from_reader(self),

            FormatOption::CSV => csv_parse(self, None),

            FormatOption::TSV => csv_parse(self, Some(&'\t')),
            FormatOption::ARROW => todo!(),
            FormatOption::TABLED => todo!(),
        };
    }
}
impl IntoSerde for DataFrame {
    // this sucks
    fn to_serde(&mut self) -> Result<Value, Error> {
        let mut buffer = Vec::new();

        JsonWriter::new(&mut buffer)
            .with_json_format(JsonFormat::Json)
            .finish(self)
            .unwrap();

        let json_string = String::from_utf8(buffer).unwrap();
        let converted = serde_json::from_str(&json_string).unwrap();
        Ok(converted)
    }
}

impl FromSerde for serde_json::Value {
    fn to_table(&self) -> Result<String, ()> {
        value_to_table(&self, None)
    }
    fn to_json_string(&self) -> String {
        self.to_string()
    }
    fn to_format_option(&self, format: &FormatOption) -> Result<String, ()> {
        match format {
            FormatOption::JSON => Ok(self.to_json_string()),
            FormatOption::CSV => todo!(),
            FormatOption::TSV => todo!(),
            FormatOption::ARROW => todo!(),
            FormatOption::TABLED => value_to_table(self, None),
        }
    }
}

use std::io::{BufReader, Read};

use polars::prelude::{DataFrame, JsonFormat, JsonWriter};
use polars_io::SerWriter;
use serde_json::{ Value};
use anyhow::{Error};

use crate::converters::csv_parse;
use crate::parsers::universal::simple_table_parse;

use super::tabled::value_to_table;

#[derive(Debug)]
pub enum FormatOption {
    JSON,
    CSV,
    TSV,
    ARROW,
    TABLED,
    PS,
    NONE,
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
            FormatOption::JSON => serde_json::from_reader(self).map_err(Error::msg),

            FormatOption::CSV => csv_parse(self, None),

            FormatOption::TSV => csv_parse(self, Some(&'\t')),
            FormatOption::ARROW => todo!(),
            FormatOption::TABLED => todo!(),
            FormatOption::NONE => todo!(),
            FormatOption::PS => {
                let mut buffer = String::new();
                self.read_to_string(&mut buffer).unwrap();
                let as_json = simple_table_parse(buffer)?;
                info!("Table converted to json is: \n{:#?}",as_json);
                Ok(as_json)
            },
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
            FormatOption::NONE => Err(()),
            FormatOption::TABLED => value_to_table(self, None),
            FormatOption::PS => todo!(),
        }
    }
}

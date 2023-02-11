use std::io::{BufReader, Read};

use polars::prelude::{DataFrame, JsonFormat, JsonWriter};
use polars_io::SerWriter;
use serde_json::{Error, Value};

use crate::converters::csv_parse;

use super::structured::FormatOption;

pub trait SerInputAdapter {
    fn to_serde_json(&mut self) -> Result<Value, Error>;
}
pub trait SerOutputAdapter {
    fn parse_as(&mut self, format: &FormatOption) -> Result<Value, Error>;
}

impl<T> SerOutputAdapter for BufReader<T>
where
    T: Read,
{
    fn parse_as(&mut self, format: &FormatOption) -> Result<Value, Error> {
        return match format {
            FormatOption::JSON => serde_json::from_reader(self),

            FormatOption::CSV => csv_parse(self, None),

            FormatOption::TSV => csv_parse(self, Some(&'\t')),
            FormatOption::ARROW => todo!(),
            FormatOption::TABLED => todo!(),
        };
    }
}
impl SerInputAdapter for DataFrame {
    fn to_serde_json(&mut self) -> Result<Value, Error> {
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

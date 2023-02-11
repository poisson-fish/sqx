use std::io::{BufRead, Cursor, Read};

use polars::prelude::{CsvReader, DataFrame};
use polars_io::SerReader;
use serde_json::{Error, Value};

use crate::traits::ser_adapter::IntoSerde;

pub fn csv_parse<R: Read>(
    buf_reader: &mut std::io::BufReader<R>,
    delim_override: Option<&char>,
) -> Result<Value, Error> {
    let buf_bytes = buf_reader.fill_buf().unwrap();
    let mmap = Cursor::new(buf_bytes);
    if let Some(delim_over) = delim_override {
        let mut df: DataFrame = CsvReader::new(mmap)
            .with_delimiter(*delim_over as u8)
            .has_header(true)
            .finish()
            .expect(
                format!(
                    "Could not parse CSV with delimiter '{}' into DataFrame",
                    delim_over
                )
                .as_str(),
            );
        let result = df.to_serde();
        debug!("DataFrame read to {:#?}", result);
        result
    } else {
        let mut df: DataFrame = CsvReader::new(mmap)
            .finish()
            .expect("Could not parse CSV with delimiter ',' into DataFrame");
        let result = df.to_serde();
        debug!("DataFrame read to {:#?}", result);
        result
    }
}

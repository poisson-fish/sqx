use anyhow::{Error,anyhow};
use serde_json::{Value, Map};

//-----------------------------------------------------------------------------
// This code is adapted from the JC (JsonConvert) Project, go check them out:
// https://github.com/kellyjonbrazil/jc
// code adapted from Conor Heine at:
// https://gist.github.com/cahna/43a1a3ff4d075bcd71f9d7120037a501
//-----------------------------------------------------------------------------

use std::{collections::BTreeMap, str::FromStr};


pub fn simple_table_parse(data: String) -> Result<Value, Error>
{
    let mut rows = Vec::new();
    let mut data = data.lines().into_iter();

    // extract headers
    let headers: Vec<_> = match data.next() {
        Some(header_row) => header_row
            .split_whitespace()
            .map(|s| s
                .replace(" ", "_")
                .replace("%", "")
                .to_lowercase())
            .collect(),
        None => return  Err(anyhow!("Expected at least one row")),
    };

    // parse data rows
    for row in data {
        let cells: Vec<_> = row.split_whitespace().collect();
        if cells.len() == headers.len() {
            let map: BTreeMap<_, _> = headers.iter().map(|header| {
                let value = cells[headers.iter().position(|h| h == header).unwrap()].to_string();
                (header.clone(), Value::from_str(value.as_str()).unwrap_or(Value::String(value.to_owned())))
            }).collect();
            rows.push(serde_json::to_value(map).unwrap());
        }
    }

    Ok(Value::Array(rows))
}

pub fn sparse_table_parse(data: String) -> Result<Value, Error>
{
    let mut data = data.lines();
    let headers: Vec<_> = data
        .next()
        .expect("Input is empty")
        .split('|')
        .map(str::trim)
        .map(ToString::to_string)
        .collect();

    let objects: Vec<Value> = data
        .map(|line| {
            let values: Vec<_> = line.split('|').map(str::trim).map(ToString::to_string).collect();
            let mut map = Map::new();
            for (header, value) in headers.iter().zip(values) {
                map.insert( header.clone(), Value::String(value));
            }
            Value::Object(map)
        })
        .collect();

    Ok(Value::Array(objects))
}
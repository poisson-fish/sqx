use anyhow::{Error, bail};
use tabled::{builder::Builder, object::{Rows, Segment}, Alignment, ModifyObject, Style, Width, Modify};

pub fn value_to_table(json: serde_json::Value) -> Result<String, Error> {
    match json {
        serde_json::Value::Array(arr) => {
            let mut builder = Builder::default();

            for (i, row) in arr.iter().enumerate() {
                let mut column_build = vec![];
                match row {
                    serde_json::Value::Null => todo!(),
                    serde_json::Value::Bool(_) => todo!(),
                    serde_json::Value::Number(_) => todo!(),
                    serde_json::Value::String(_) => todo!(),
                    serde_json::Value::Array(_) => todo!(),
                    serde_json::Value::Object(object_to_row) => {
                        
                        let columns = object_to_row.keys();
                        builder.set_columns(columns);
                        
                        for (k,v) in object_to_row {
                            column_build.push(v.to_string());
                        }
                    }
                }
                builder.add_record(column_build);
            }
            Ok(builder
                .build()
                .with(Modify::new(Segment::all()).with(Width::truncate(32)))
                .with(Style::rounded())
                .with(Rows::new(1..).modify().with(Alignment::left()))
                .to_string())
        }
        _ => bail!("table input wasn't an array of objects"),
    }
}

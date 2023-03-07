use tabled::{
    builder::Builder,
    object::{Rows, Segment},
    Alignment, Modify, ModifyObject, Style, Width,
};

pub struct TableSettings {
    max_text_length: usize,
}

pub fn value_to_table(
    value: &serde_json::Value,
    opt_param: Option<TableSettings>,
) -> Result<String, ()> {
    let options = opt_param.unwrap_or(TableSettings {
        max_text_length: 32,
    });
    match value {
        serde_json::Value::Array(arr) => {
            let mut builder = Builder::default();

            for row in arr {
                let mut column_build = vec![];

                match row {
                    serde_json::Value::Null => todo!(),
                    serde_json::Value::Bool(_) => todo!(),
                    serde_json::Value::Number(_) => todo!(),
                    serde_json::Value::String(v) => column_build.push(v.to_string()),
                    serde_json::Value::Array(_) => todo!(),
                    serde_json::Value::Object(object_to_row) => {
                        let columns = object_to_row.keys();
                        builder.set_columns(columns);
                        for (_, v) in object_to_row {
                            column_build.push(v.to_string());
                        }
                    }
                }
                builder.add_record(column_build);
            }

            Ok(builder
                .build()
                .with(Modify::new(Segment::all()).with(Width::truncate(options.max_text_length)))
                .with(Style::rounded())
                .with(Rows::new(1..).modify().with(Alignment::left()))
                .to_string())
        }
        _ => Err(()),
    }
}

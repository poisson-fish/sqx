use super::tabled::Tabled;
use anyhow::Error;

#[derive(Debug)]
pub enum FormatOption {
    JSON,
    CSV,
    TSV,
    ARROW,
    TABLED,
}

pub trait Structured {
    fn format_to_string(&self, fmt: FormatOption) -> Result<String, Error>;
}

impl Structured for serde_json::Value {
    fn format_to_string(&self, fmt: FormatOption) -> Result<String, Error> {
        match fmt {
            FormatOption::JSON => Ok(self.to_string()),
            FormatOption::CSV => todo!(),
            FormatOption::TSV => todo!(),
            FormatOption::ARROW => todo!(),
            FormatOption::TABLED => self.value_to_table(None),
        }
    }
}

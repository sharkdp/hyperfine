use crate::util::number::Number;
use std::fmt::Display;

pub mod file_values;
pub mod product;
pub mod range_step;
pub mod sources;
pub mod tokenize;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParameterValue {
    Text(String),
    Numeric(Number),
}

impl Display for ParameterValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let str = match self {
            ParameterValue::Text(ref value) => value.clone(),
            ParameterValue::Numeric(value) => value.to_string(),
        };
        write!(f, "{str}")
    }
}

pub type ParameterNameAndValue<'a> = (&'a str, ParameterValue);

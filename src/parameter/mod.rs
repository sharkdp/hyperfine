use crate::util::number::Number;
use std::fmt::Display;

pub mod range_step;
pub mod tokenize;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParameterValue {
    Text(String),
    Numeric(Number),
}

impl Display for ParameterValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ParameterValue::Text(value) => value.fmt(f),
            ParameterValue::Numeric(value) => value.fmt(f),
        }
    }
}

pub type ParameterNameAndValue<'a> = (&'a str, ParameterValue);

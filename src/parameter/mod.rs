use crate::util::number::Number;

pub mod range_step;
pub mod tokenize;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParameterValue {
    Text(String),
    Numeric(Number),
}

impl<'a> ToString for ParameterValue {
    fn to_string(&self) -> String {
        match self {
            ParameterValue::Text(ref value) => value.clone(),
            ParameterValue::Numeric(value) => value.to_string(),
        }
    }
}

pub type ParameterNameAndValue<'a> = (&'a str, ParameterValue);

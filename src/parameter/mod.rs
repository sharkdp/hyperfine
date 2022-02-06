use crate::numeric::NumericType;

pub mod range;
pub mod tokenize;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParameterValue {
    Text(String),
    Numeric(NumericType),
}

impl<'a> ToString for ParameterValue {
    fn to_string(&self) -> String {
        match self {
            ParameterValue::Text(ref value) => value.clone(),
            ParameterValue::Numeric(value) => value.to_string(),
        }
    }
}

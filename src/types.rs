use std::convert::TryFrom;
use std::fmt;

use rust_decimal::prelude::ToPrimitive;
use rust_decimal::Decimal;
use serde::Serialize;

#[derive(Debug, Clone, Serialize, Copy, PartialEq, Eq)]
#[serde(untagged)]
pub enum NumericType {
    Int(i32),
    Decimal(Decimal),
}

impl fmt::Display for NumericType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            NumericType::Int(i) => fmt::Display::fmt(&i, f),
            NumericType::Decimal(i) => fmt::Display::fmt(&i, f),
        }
    }
}

impl From<i32> for NumericType {
    fn from(x: i32) -> NumericType {
        NumericType::Int(x)
    }
}

impl From<Decimal> for NumericType {
    fn from(x: Decimal) -> NumericType {
        NumericType::Decimal(x)
    }
}

impl TryFrom<NumericType> for usize {
    type Error = ();

    fn try_from(numeric: NumericType) -> Result<Self, Self::Error> {
        match numeric {
            NumericType::Int(i) => usize::try_from(i).map_err(|_| ()),
            NumericType::Decimal(d) => match d.to_u64() {
                Some(u) => usize::try_from(u).map_err(|_| ()),
                None => Err(()),
            },
        }
    }
}

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

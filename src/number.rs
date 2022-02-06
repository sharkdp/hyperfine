use std::convert::TryFrom;
use std::fmt;

use rust_decimal::prelude::ToPrimitive;
use rust_decimal::Decimal;
use serde::Serialize;

#[derive(Debug, Clone, Serialize, Copy, PartialEq, Eq)]
#[serde(untagged)]
pub enum Number {
    Int(i32),
    Decimal(Decimal),
}

impl fmt::Display for Number {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Number::Int(i) => fmt::Display::fmt(&i, f),
            Number::Decimal(i) => fmt::Display::fmt(&i, f),
        }
    }
}

impl From<i32> for Number {
    fn from(x: i32) -> Number {
        Number::Int(x)
    }
}

impl From<Decimal> for Number {
    fn from(x: Decimal) -> Number {
        Number::Decimal(x)
    }
}

impl TryFrom<Number> for usize {
    type Error = ();

    fn try_from(numeric: Number) -> Result<Self, Self::Error> {
        match numeric {
            Number::Int(i) => usize::try_from(i).map_err(|_| ()),
            Number::Decimal(d) => match d.to_u64() {
                Some(u) => usize::try_from(u).map_err(|_| ()),
                None => Err(()),
            },
        }
    }
}

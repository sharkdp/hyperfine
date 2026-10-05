use std::fmt::{self, Display, Formatter};

use serde::Serialize;

#[derive(Debug, Clone, Serialize, PartialEq)]
enum MetricPrefix {
    Nano,
    Micro,
    Milli,
    None,
}

impl Display for MetricPrefix {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            MetricPrefix::Nano => write!(f, "n"),
            MetricPrefix::Micro => write!(f, "µ"),
            MetricPrefix::Milli => write!(f, "m"),
            MetricPrefix::None => Ok(()),
        }
    }
}

#[derive(Debug, Clone, Serialize, PartialEq)]
enum BinaryPrefix {
    None,
    Kibi,
    Mebi,
    Gibi,
    Tebi,
}

impl Display for BinaryPrefix {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            BinaryPrefix::None => Ok(()),
            BinaryPrefix::Kibi => write!(f, "Ki"),
            BinaryPrefix::Mebi => write!(f, "Mi"),
            BinaryPrefix::Gibi => write!(f, "Gi"),
            BinaryPrefix::Tebi => write!(f, "Ti"),
        }
    }
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub enum Unit {
    Second(MetricPrefix),
    Byte(BinaryPrefix),
}

pub const SECOND: Unit = Unit::Second(MetricPrefix::None);
pub const MILLISECOND: Unit = Unit::Second(MetricPrefix::Milli);
pub const MICROSECOND: Unit = Unit::Second(MetricPrefix::Micro);
pub const BYTE: Unit = Unit::Byte(BinaryPrefix::None);

impl Display for Unit {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            Unit::Second(prefix) => write!(f, "{prefix}s",),
            Unit::Byte(prefix) => write!(f, "{prefix}B"),
        }
    }
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct Quantity<Value> {
    value: Value,
    unit: Unit,
}

impl<Value> Quantity<Value> {
    pub fn new(value: Value, unit: Unit) -> Self {
        Self { value, unit }
    }
}

impl<Value: Display> Display for Quantity<Value> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{} {}", self.value, self.unit)
    }
}

pub fn mean(values: &[Quantity<f64>]) -> Quantity<f64> {
    statistical::mean(values).unwrap()
}

pub type Time = Quantity<f64>;
pub type Memory = Quantity<u64>;

#[test]
fn basic() {
    let time = Time {
        value: 123.4,
        unit: Unit::Second(MetricPrefix::Milli),
    };
    assert_eq!(time.to_string(), "123.4 ms");

    let peak_memory_usage = Memory {
        value: 8,
        unit: Unit::Byte(BinaryPrefix::Kibi),
    };
    assert_eq!(peak_memory_usage.to_string(), "8 KiB");
}

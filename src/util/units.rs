//! This module contains common units.

use crate::quantity::{microsecond, millisecond, second, Time, TimeQuantity};

/// Supported time units
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Unit {
    Second,
    MilliSecond,
    MicroSecond,
}

impl Unit {
    /// The abbreviation of the Unit.
    pub fn short_name(self) -> String {
        match self {
            Unit::Second => String::from("s"),
            Unit::MilliSecond => String::from("ms"),
            Unit::MicroSecond => String::from("µs"),
        }
    }

    /// Returns the Second value formatted for the Unit.
    pub fn format(self, value: Time) -> String {
        match self {
            Unit::Second => format!("{value:.3}", value = value.value_in::<second>()), // TODO: use .to_string on Time?
            Unit::MilliSecond => format!("{value:.1}", value = value.value_in::<millisecond>()),
            Unit::MicroSecond => format!("{value:.1}", value = value.value_in::<microsecond>()),
        }
    }
}

#[test]
fn test_unit_short_name() {
    assert_eq!("s", Unit::Second.short_name());
    assert_eq!("ms", Unit::MilliSecond.short_name());
    assert_eq!("µs", Unit::MicroSecond.short_name());
}

// Note - the values are rounded when formatted.
#[test]
fn test_unit_format() {
    let value = Time::from_seconds(123.456789);
    assert_eq!("123.457", Unit::Second.format(value));
    assert_eq!("123456.8", Unit::MilliSecond.format(value));

    assert_eq!(
        "1234.6",
        Unit::MicroSecond.format(Time::from_seconds(0.00123456))
    );
}

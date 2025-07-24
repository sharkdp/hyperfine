//! This module contains common units.

pub type Scalar = f64;

/// Type alias for unit of time
pub type Second = Scalar;

/// Supported time units
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Unit {
    Second,
    MilliSecond,
    MicroSecond,
}

impl Unit {
    /// The abbreviation of the Unit.
    #[must_use]
    pub const fn short_name(self) -> &'static str {
        match self {
            Unit::Second => "s",
            Unit::MilliSecond => "ms",
            Unit::MicroSecond => "µs",
        }
    }

    /// Returns the Second value formatted for the Unit.
    #[must_use]
    pub fn format(self, value: Second) -> String {
        match self {
            Unit::Second => format!("{value:.3}"),
            Unit::MilliSecond => format!("{:.1}", value * 1e3),
            Unit::MicroSecond => format!("{:.1}", value * 1e6),
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
    let value: Second = 123.456789;
    assert_eq!("123.457", Unit::Second.format(value));
    assert_eq!("123456.8", Unit::MilliSecond.format(value));

    assert_eq!("1234.6", Unit::MicroSecond.format(0.00123456));
}

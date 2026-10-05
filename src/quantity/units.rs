//! This module contains common units.

/// Supported time units
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TimeUnit {
    Second,
    MilliSecond,
    MicroSecond,
    Minute,
    Hour,
}

impl TimeUnit {
    /// The abbreviation of the TimeUnit.
    pub fn short_name(self) -> String {
        match self {
            TimeUnit::Second => String::from("s"),
            TimeUnit::MilliSecond => String::from("ms"),
            TimeUnit::MicroSecond => String::from("µs"),
            TimeUnit::Minute => String::from("min"),
            TimeUnit::Hour => String::from("h"),
        }
    }

    /// Returns the value in seconds formatted for the TimeUnit.
    pub fn format(self, value: f64) -> String {
        match self {
            TimeUnit::Second => format!("{value:.3}"),
            TimeUnit::MilliSecond => format!("{:.1}", value * 1e3),
            TimeUnit::MicroSecond => format!("{:.1}", value * 1e6),
            TimeUnit::Minute => format!("{:.1}", value / 60.0),
            TimeUnit::Hour => format!("{:.1}", value / 3600.0),
        }
    }
}

#[test]
fn test_unit_short_name() {
    assert_eq!("s", TimeUnit::Second.short_name());
    assert_eq!("ms", TimeUnit::MilliSecond.short_name());
    assert_eq!("µs", TimeUnit::MicroSecond.short_name());
    assert_eq!("min", TimeUnit::Minute.short_name());
    assert_eq!("h", TimeUnit::Hour.short_name());
}

// Note - the values are rounded when formatted.
#[test]
fn test_unit_format() {
    let value: f64 = 123.456789;
    assert_eq!("123.457", TimeUnit::Second.format(value));
    assert_eq!("123456.8", TimeUnit::MilliSecond.format(value));

    assert_eq!("1234.6", TimeUnit::MicroSecond.format(0.00123456));
    assert_eq!("2.1", TimeUnit::Minute.format(value));
    assert_eq!("0.0", TimeUnit::Hour.format(value));
}

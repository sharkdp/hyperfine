use hyperfine::types::Second;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Unit {
    Second,
    MilliSecond,
}

impl Unit {
    /// The abbreviation of the Unit.
    pub fn short_name(&self) -> String {
        match *self {
            Unit::Second => String::from("s"),
            Unit::MilliSecond => String::from("ms"),
        }
    }

    /// Returns the Second value formatted for the Unit.
    pub fn format(&self, value: Second) -> String {
        match *self {
            Unit::Second => format!("{:.3}", value),
            Unit::MilliSecond => format!("{:.1}", value * 1e3),
        }
    }
}

/// Format the given duration as a string. The output-unit can be enforced by setting `unit` to
/// `Some(target_unit)`. If `unit` is `None`, it will be determined automatically.
pub fn format_duration(duration: Second, unit: Option<Unit>) -> String {
    match format_duration_unit(duration, unit) {
        (duration_fmt, _) => duration_fmt,
    }
}

/// Like `format_duration`, but returns the target unit as well.
pub fn format_duration_unit(duration: Second, unit: Option<Unit>) -> (String, Unit) {
    let (out_str, out_unit) = format_duration_value(duration, unit);

    (format!("{} {}", out_str, out_unit.short_name()), out_unit)
}

/// Like `format_duration`, but returns the target unit as well.
pub fn format_duration_value(duration: Second, unit: Option<Unit>) -> (String, Unit) {
    if (duration < 1.0 && unit.is_none()) || unit == Some(Unit::MilliSecond) {
        (Unit::MilliSecond.format(duration), Unit::MilliSecond)
    } else {
        (Unit::Second.format(duration), Unit::Second)
    }
}

#[test]
fn test_unit_short_name() {
    assert_eq!("s", Unit::Second.short_name());
    assert_eq!("ms", Unit::MilliSecond.short_name());
}

// Note - the values are rounded when formatted.
#[test]
fn test_unit_format() {
    let value: Second = 123.456789;
    assert_eq!("123.457", Unit::Second.format(value));
    assert_eq!("123456.8", Unit::MilliSecond.format(value));
}

#[test]
fn test_format_duration_unit_basic() {
    let (out_str, out_unit) = format_duration_unit(1.3, None);

    assert_eq!("1.300 s", out_str);
    assert_eq!(Unit::Second, out_unit);

    let (out_str, out_unit) = format_duration_unit(1.0, None);

    assert_eq!("1.000 s", out_str);
    assert_eq!(Unit::Second, out_unit);

    let (out_str, out_unit) = format_duration_unit(0.999, None);

    assert_eq!("999.0 ms", out_str);
    assert_eq!(Unit::MilliSecond, out_unit);

    let (out_str, out_unit) = format_duration_unit(0.0, None);

    assert_eq!("0.0 ms", out_str);
    assert_eq!(Unit::MilliSecond, out_unit);

    let (out_str, out_unit) = format_duration_unit(1000.0, None);

    assert_eq!("1000.000 s", out_str);
    assert_eq!(Unit::Second, out_unit);
}

#[test]
fn test_format_duration_unit_with_unit() {
    let (out_str, out_unit) = format_duration_unit(1.3, Some(Unit::Second));

    assert_eq!("1.300 s", out_str);
    assert_eq!(Unit::Second, out_unit);

    let (out_str, out_unit) = format_duration_unit(1.3, Some(Unit::MilliSecond));

    assert_eq!("1300.0 ms", out_str);
    assert_eq!(Unit::MilliSecond, out_unit);
}

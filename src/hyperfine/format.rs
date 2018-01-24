use hyperfine::internal::Second;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Unit {
    Second,
    MilliSecond,
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
    if (duration < 1.0 && unit.is_none()) || unit == Some(Unit::MilliSecond) {
        (format!("{:.1} ms", duration * 1e3), Unit::MilliSecond)
    } else {
        (format!("{:.3} s", duration), Unit::Second)
    }
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

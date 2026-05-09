use std::fmt::Display;

use crate::util::units::{Second, Unit};

/// Format the given duration as a string. The output-unit can be enforced by setting `unit` to
/// `Some(target_unit)`. If `unit` is `None`, it will be determined automatically.
pub fn format_duration(duration: Second, unit: Option<Unit>) -> String {
    let (duration_fmt, _) = format_duration_unit(duration, unit);
    duration_fmt
}

/// Like `format_duration`, but returns the target unit as well.
pub fn format_duration_unit(duration: Second, unit: Option<Unit>) -> (String, Unit) {
    let (out_str, out_unit) = format_duration_value(duration, unit);

    (format!("{} {}", out_str, out_unit.short_name()), out_unit)
}

/// Like `format_duration`, but returns the target unit as well.
pub fn format_duration_value(duration: Second, unit: Option<Unit>) -> (String, Unit) {
    if (duration < 0.001 && unit.is_none()) || unit == Some(Unit::MicroSecond) {
        (Unit::MicroSecond.format(duration), Unit::MicroSecond)
    } else if (duration < 1.0 && unit.is_none()) || unit == Some(Unit::MilliSecond) {
        (Unit::MilliSecond.format(duration), Unit::MilliSecond)
    } else {
        (Unit::Second.format(duration), Unit::Second)
    }
}

/// Wrapper to format memory sizes as a string.
pub struct BytesFormat(pub u64);

impl Display for BytesFormat {
    #[expect(clippy::cast_precision_loss)]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let bytes = self.0;
        if bytes < 100_000 {
            return write!(f, "{bytes} b");
        }
        let bytes = bytes as f64 / 1000.0;
        if bytes < 10.0 {
            return write!(f, "{bytes:.3} kb");
        }
        if bytes < 100.0 {
            return write!(f, "{bytes:.2} kb");
        }
        if bytes < 1000.0 {
            return write!(f, "{bytes:.1} kb");
        }
        let bytes = bytes / 1000.0;
        if bytes < 10.0 {
            return write!(f, "{bytes:.3} MB");
        }
        if bytes < 100.0 {
            return write!(f, "{bytes:.2} MB");
        }
        if bytes < 1000.0 {
            return write!(f, "{bytes:.1} MB");
        }
        let bytes = bytes / 1000.0;
        if bytes < 10.0 {
            return write!(f, "{bytes:.3} GB");
        }
        if bytes < 100.0 {
            return write!(f, "{bytes:.2} GB");
        }
        if bytes < 1000.0 {
            return write!(f, "{bytes:.1} GB");
        }
        let bytes = bytes / 1000.0;
        write!(f, "{bytes:.0} TB")
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

    let (out_str, out_unit) = format_duration_unit(0.0005, None);

    assert_eq!("500.0 µs", out_str);
    assert_eq!(Unit::MicroSecond, out_unit);

    let (out_str, out_unit) = format_duration_unit(0.0, None);

    assert_eq!("0.0 µs", out_str);
    assert_eq!(Unit::MicroSecond, out_unit);

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

    let (out_str, out_unit) = format_duration_unit(1.3, Some(Unit::MicroSecond));

    assert_eq!("1300000.0 µs", out_str);
    assert_eq!(Unit::MicroSecond, out_unit);
}

#[test]
fn test_format_bytes() {
    assert_eq!("0 b", format!("{}", BytesFormat(0)));
    assert_eq!("42 b", format!("{}", BytesFormat(42)));
    assert_eq!("10240 b", format!("{}", BytesFormat(10240)));
    assert_eq!("102.4 kb", format!("{}", BytesFormat(102400)));
    assert_eq!("102.4 MB", format!("{}", BytesFormat(102400000)));
    assert_eq!("1.024 GB", format!("{}", BytesFormat(1024000000)));
    assert_eq!("18446744 TB", format!("{}", BytesFormat(u64::MAX)));
}

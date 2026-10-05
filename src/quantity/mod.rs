use std::marker::PhantomData;

pub use uom::num_traits::Zero;
pub use uom::si::f64::{Information, Ratio, Time};
pub use uom::si::information::byte;
pub use uom::si::ratio::ratio;
pub use uom::si::time::{microsecond, nanosecond, second};

pub use units::TimeUnit;

pub mod statistics;
mod units;

pub const fn const_time_from_seconds(value: f64) -> Time {
    // Quantity::new in uom is not yet const: https://docs.rs/uom/0.36.0/uom/si/struct.Quantity.html
    Time {
        dimension: PhantomData,
        units: PhantomData,
        value,
    }
}

/// Format the given duration as a string. The output-unit can be enforced by setting `unit` to
/// `Some(target_unit)`. If `unit` is `None`, it will be determined automatically.
pub fn format_duration(duration: f64, unit: Option<TimeUnit>) -> String {
    let (duration_fmt, _) = format_duration_unit(duration, unit);
    duration_fmt
}

/// Like `format_duration`, but returns the target unit as well.
pub fn format_duration_unit(duration: f64, unit: Option<TimeUnit>) -> (String, TimeUnit) {
    let (out_str, out_unit) = format_duration_value(duration, unit);

    (format!("{} {}", out_str, out_unit.short_name()), out_unit)
}

/// Like `format_duration`, but returns the target unit as well.
pub fn format_duration_value(duration: f64, unit: Option<TimeUnit>) -> (String, TimeUnit) {
    if (duration < 0.001 && unit.is_none()) || unit == Some(TimeUnit::MicroSecond) {
        (
            TimeUnit::MicroSecond.format(duration),
            TimeUnit::MicroSecond,
        )
    } else if (duration < 1.0 && unit.is_none()) || unit == Some(TimeUnit::MilliSecond) {
        (
            TimeUnit::MilliSecond.format(duration),
            TimeUnit::MilliSecond,
        )
    } else {
        let unit = unit.unwrap_or(TimeUnit::Second);
        (unit.format(duration), unit)
    }
}

#[cfg(test)]
use uom::si::{information::kibibyte, time::millisecond};

#[test]
fn test_time() {
    let time = Time::new::<millisecond>(123.4);
    assert_eq!(time.get::<millisecond>(), 123.4);

    let time_s = time.get::<second>();
    approx::assert_relative_eq!(time_s, 0.1234);

    let time_us = time.get::<microsecond>();
    approx::assert_relative_eq!(time_us, 123400.0);
}

#[test]
fn test_information() {
    use uom::si::information::pebibyte;

    let information = Information::new::<kibibyte>(8.);
    assert_eq!(information.get::<byte>(), 8192.);

    let information_kib = information.get::<kibibyte>();
    assert_eq!(information_kib, 8.);

    let largest_exactly_representable = Information::new::<byte>(9_007_199_254_740_992.);
    assert_eq!(largest_exactly_representable.get::<pebibyte>(), 8.);
}

#[test]
fn test_format_duration_unit_basic() {
    let (out_str, out_unit) = format_duration_unit(1.3, None);

    assert_eq!("1.300 s", out_str);
    assert_eq!(TimeUnit::Second, out_unit);

    let (out_str, out_unit) = format_duration_unit(1.0, None);

    assert_eq!("1.000 s", out_str);
    assert_eq!(TimeUnit::Second, out_unit);

    let (out_str, out_unit) = format_duration_unit(0.999, None);

    assert_eq!("999.0 ms", out_str);
    assert_eq!(TimeUnit::MilliSecond, out_unit);

    let (out_str, out_unit) = format_duration_unit(0.0005, None);

    assert_eq!("500.0 µs", out_str);
    assert_eq!(TimeUnit::MicroSecond, out_unit);

    let (out_str, out_unit) = format_duration_unit(0.0, None);

    assert_eq!("0.0 µs", out_str);
    assert_eq!(TimeUnit::MicroSecond, out_unit);

    let (out_str, out_unit) = format_duration_unit(1000.0, None);

    assert_eq!("1000.000 s", out_str);
    assert_eq!(TimeUnit::Second, out_unit);
}

#[test]
fn test_format_duration_unit_with_unit() {
    let (out_str, out_unit) = format_duration_unit(1.3, Some(TimeUnit::Second));

    assert_eq!("1.300 s", out_str);
    assert_eq!(TimeUnit::Second, out_unit);

    let (out_str, out_unit) = format_duration_unit(1.3, Some(TimeUnit::MilliSecond));

    assert_eq!("1300.0 ms", out_str);
    assert_eq!(TimeUnit::MilliSecond, out_unit);

    let (out_str, out_unit) = format_duration_unit(1.3, Some(TimeUnit::MicroSecond));

    assert_eq!("1300000.0 µs", out_str);
    assert_eq!(TimeUnit::MicroSecond, out_unit);
}

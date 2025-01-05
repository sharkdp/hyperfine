use crate::{
    quantity::{second, Time},
    util::units::TimeUnit,
};

/// Format the given duration as a string. The output-unit can be enforced by setting `unit` to
/// `Some(target_unit)`. If `unit` is `None`, it will be determined automatically.
pub fn format_duration(duration: Time, time_unit: Option<TimeUnit>) -> String {
    let (duration_fmt, _) = format_duration_unit(duration, time_unit);
    duration_fmt
}

/// Like `format_duration`, but returns the target unit as well.
pub fn format_duration_unit(duration: Time, time_unit: Option<TimeUnit>) -> (String, TimeUnit) {
    let (out_str, out_unit) = format_duration_value(duration, time_unit);

    (format!("{} {}", out_str, out_unit.short_name()), out_unit)
}

/// Like `format_duration`, but returns the target unit as well.
pub fn format_duration_value(duration: Time, time_unit: Option<TimeUnit>) -> (String, TimeUnit) {
    if (duration < Time::new::<second>(0.001) && time_unit.is_none())
        || time_unit == Some(TimeUnit::MicroSecond)
    {
        (
            TimeUnit::MicroSecond.format(duration),
            TimeUnit::MicroSecond,
        )
    } else if (duration < Time::new::<second>(1.0) && time_unit.is_none())
        || time_unit == Some(TimeUnit::MilliSecond)
    {
        (
            TimeUnit::MilliSecond.format(duration),
            TimeUnit::MilliSecond,
        )
    } else {
        let time_unit = time_unit.unwrap_or(TimeUnit::Second);
        (time_unit.format(duration), time_unit)
    }
}

#[test]
fn test_format_duration_unit_basic() {
    let (out_str, out_unit) = format_duration_unit(Time::new::<second>(1.3), None);

    assert_eq!("1.300 s", out_str);
    assert_eq!(TimeUnit::Second, out_unit);

    let (out_str, out_unit) = format_duration_unit(Time::new::<second>(1.0), None);

    assert_eq!("1.000 s", out_str);
    assert_eq!(TimeUnit::Second, out_unit);

    let (out_str, out_unit) = format_duration_unit(Time::new::<second>(0.999), None);

    assert_eq!("999.0 ms", out_str);
    assert_eq!(TimeUnit::MilliSecond, out_unit);

    let (out_str, out_unit) = format_duration_unit(Time::new::<second>(0.0005), None);

    assert_eq!("500.0 µs", out_str);
    assert_eq!(TimeUnit::MicroSecond, out_unit);

    let (out_str, out_unit) = format_duration_unit(Time::new::<second>(0.), None);

    assert_eq!("0.0 µs", out_str);
    assert_eq!(TimeUnit::MicroSecond, out_unit);

    let (out_str, out_unit) = format_duration_unit(Time::new::<second>(1000.0), None);

    assert_eq!("1000.000 s", out_str);
    assert_eq!(TimeUnit::Second, out_unit);
}

#[test]
fn test_format_duration_unit_with_unit() {
    let (out_str, out_unit) =
        format_duration_unit(Time::new::<second>(1.3), Some(TimeUnit::Second));

    assert_eq!("1.300 s", out_str);
    assert_eq!(TimeUnit::Second, out_unit);

    let (out_str, out_unit) =
        format_duration_unit(Time::new::<second>(1.3), Some(TimeUnit::MilliSecond));

    assert_eq!("1300.0 ms", out_str);
    assert_eq!(TimeUnit::MilliSecond, out_unit);

    let (out_str, out_unit) =
        format_duration_unit(Time::new::<second>(1.3), Some(TimeUnit::MicroSecond));

    assert_eq!("1300000.0 µs", out_str);
    assert_eq!(TimeUnit::MicroSecond, out_unit);
}

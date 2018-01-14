use hyperfine::internal::Second;

#[derive(PartialEq)]
pub enum Unit {
    Auto,
    Second,
    MilliSecond,
}

pub fn format_duration(duration: Second, unit: Unit) -> String {
    match format_duration_unit(duration, unit) {
        (duration_fmt, _) => duration_fmt,
    }
}

pub fn format_duration_unit(duration: Second, unit: Unit) -> (String, Unit) {
    if (duration < 1.0 && unit == Unit::Auto) || unit == Unit::MilliSecond {
        (format!("{:.1} ms", duration * 1e3), Unit::MilliSecond)
    } else {
        (format!("{:.3} s", duration), Unit::Second)
    }
}

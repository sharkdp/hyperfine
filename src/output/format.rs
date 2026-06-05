use crate::util::units::{Second, Unit};

/// Unit for memory formatting
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MemoryUnit {
    Byte,
    KiloByte,
    MegaByte,
    GigaByte,
}

impl MemoryUnit {
    fn short_name(&self) -> &'static str {
        match self {
            MemoryUnit::Byte => "B",
            MemoryUnit::KiloByte => "KB",
            MemoryUnit::MegaByte => "MB",
            MemoryUnit::GigaByte => "GB",
        }
    }

    fn bytes_per_unit(&self) -> f64 {
        match self {
            MemoryUnit::Byte => 1.0,
            MemoryUnit::KiloByte => 1024.0,
            MemoryUnit::MegaByte => 1024.0 * 1024.0,
            MemoryUnit::GigaByte => 1024.0 * 1024.0 * 1024.0,
        }
    }
}

/// Format memory size in bytes to a human-readable string with automatic unit selection.
pub fn format_memory(bytes: u64) -> String {
    format_memory_value(bytes, None)
}

/// Format memory size with optional right-padding to specified width.
pub fn format_memory_value(bytes: u64, width: Option<usize>) -> String {
    let bytes_f = bytes as f64;

    let (value, unit) = if bytes_f >= MemoryUnit::GigaByte.bytes_per_unit() {
        (
            bytes_f / MemoryUnit::GigaByte.bytes_per_unit(),
            MemoryUnit::GigaByte,
        )
    } else if bytes_f >= MemoryUnit::MegaByte.bytes_per_unit() {
        (
            bytes_f / MemoryUnit::MegaByte.bytes_per_unit(),
            MemoryUnit::MegaByte,
        )
    } else if bytes_f >= MemoryUnit::KiloByte.bytes_per_unit() {
        (
            bytes_f / MemoryUnit::KiloByte.bytes_per_unit(),
            MemoryUnit::KiloByte,
        )
    } else {
        (bytes_f, MemoryUnit::Byte)
    };

    let formatted = format!("{:.1} {}", value, unit.short_name());
    match width {
        Some(w) => format!("{:>width$}", formatted, width = w),
        None => formatted,
    }
}

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
fn test_format_memory() {
    assert_eq!(format_memory(0), "0.0 B");
    assert_eq!(format_memory(512), "512.0 B");
    assert_eq!(format_memory(1023), "1023.0 B");
    assert_eq!(format_memory(1024), "1.0 KB");
    assert_eq!(format_memory(1536), "1.5 KB");
    assert_eq!(format_memory(1024 * 1024), "1.0 MB");
    assert_eq!(format_memory(44_564_480), "42.5 MB");
    assert_eq!(format_memory(1024 * 1024 * 1024), "1.0 GB");
    assert_eq!(format_memory(2 * 1024 * 1024 * 1024), "2.0 GB");
}

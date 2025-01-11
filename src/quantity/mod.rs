use core::f64;
use std::marker::PhantomData;

use serde::ser::SerializeStruct;
use serde::Serializer;
use uom::{si, Conversion};

pub use uom::si::information::{byte, kibibyte};
pub use uom::si::ratio::ratio;
pub use uom::si::time::{hour, microsecond, millisecond, minute, nanosecond, second};

pub use si::f64::Ratio;
pub use si::f64::Time;
pub use si::u64::Information;

pub use units::TimeUnit;

mod units;

pub trait TimeQuantity {
    fn zero() -> Time {
        Time::new::<second>(0.0)
    }

    fn value_in<U>(&self, u: U) -> f64
    where
        U: si::time::Unit + Conversion<f64, T = f64>;

    fn format_value_in(&self, time_unit: TimeUnit, precision: usize) -> String;
    fn format_auto(&self, time_unit: Option<TimeUnit>) -> String;
    fn format_auto_with_unit(&self, time_unit: Option<TimeUnit>) -> (String, TimeUnit);
    fn format_value_auto(&self, time_unit: Option<TimeUnit>) -> (String, TimeUnit);
}

impl TimeQuantity for Time {
    fn value_in<U>(&self, _u: U) -> f64
    where
        U: si::time::Unit + Conversion<f64, T = f64>,
    {
        self.get::<U>()
    }

    fn format_value_in(&self, u: TimeUnit, precision: usize) -> String {
        u.format(*self, precision)
    }

    /// Format the given duration as a string. The output-unit can be enforced by setting `unit` to
    /// `Some(target_unit)`. If `unit` is `None`, it will be determined automatically.
    fn format_auto(&self, time_unit: Option<TimeUnit>) -> String {
        self.format_auto_with_unit(time_unit).0
    }

    /// Like `format_duration`, but returns the target unit as well.
    fn format_auto_with_unit(&self, time_unit: Option<TimeUnit>) -> (String, TimeUnit) {
        let (out_str, out_unit) = self.format_value_auto(time_unit);

        (format!("{} {}", out_str, out_unit.short_name()), out_unit)
    }

    /// Like `format_duration`, but returns the target unit as well.
    fn format_value_auto(&self, time_unit: Option<TimeUnit>) -> (String, TimeUnit) {
        let (time_unit, precision) = if (*self < Time::new::<millisecond>(1.0)
            && time_unit.is_none())
            || time_unit == Some(TimeUnit::MicroSecond)
        {
            (TimeUnit::MicroSecond, 1)
        } else if (*self < Time::new::<second>(1.0) && time_unit.is_none())
            || time_unit == Some(TimeUnit::MilliSecond)
        {
            (TimeUnit::MilliSecond, 1)
        } else {
            let time_unit = time_unit.unwrap_or(TimeUnit::Second);
            let precision = if time_unit == TimeUnit::Second { 3 } else { 1 };

            (time_unit, precision)
        };

        (self.format_value_in(time_unit, precision), time_unit)
    }
}

pub const fn const_time_from_seconds(value: f64) -> Time {
    // Quantity::new in uom is not yet const: https://docs.rs/uom/0.36.0/uom/si/struct.Quantity.html
    Time {
        dimension: PhantomData,
        units: PhantomData,
        value,
    }
}

pub trait InformationQuantity {
    fn zero() -> Information {
        Information::new::<byte>(0)
    }

    fn value_in<U>(&self, u: U) -> u64
    where
        U: si::information::Unit + Conversion<u64, T = uom::num_rational::Ratio<u64>>;

    fn to_string<U>(&self, u: U) -> String
    where
        U: si::information::Unit + Conversion<u64, T = uom::num_rational::Ratio<u64>>;
}

impl InformationQuantity for Information {
    fn value_in<U>(&self, _u: U) -> u64
    where
        U: si::information::Unit + Conversion<u64, T = uom::num_rational::Ratio<u64>>,
    {
        self.get::<U>()
    }

    fn to_string<U>(&self, u: U) -> String
    where
        U: si::information::Unit + Conversion<u64, T = uom::num_rational::Ratio<u64>>,
    {
        format!(
            "{}",
            self.into_format_args(u, uom::fmt::DisplayStyle::Abbreviation)
        )
    }
}

pub fn serialize_time<S>(t: &Time, s: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let mut state = s.serialize_struct("Time", 3)?;
    state.serialize_field("value", &t.value)?;
    state.serialize_field("unit", "second")?;
    state.end()
}

pub fn serialize_information<S>(i: &Information, s: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let mut state = s.serialize_struct("Information", 3)?;
    state.serialize_field("value", &i.value)?;
    state.serialize_field("unit", "byte")?;
    state.end()
}

macro_rules! quantity_fn {
    ($name:ident, $unwrapped_values:ident, $body:expr) => {
        pub fn $name(values: &[Time]) -> Time {
            let $unwrapped_values: Vec<_> = values.iter().map(|q| q.value_in(second)).collect();
            let result_value = $body;

            Time::new::<second>(result_value)
        }
    };
}

quantity_fn!(mean, values, statistical::mean(&values));
quantity_fn!(median, values, statistical::median(&values));
quantity_fn!(min, values, crate::util::min_max::min(&values));
quantity_fn!(max, values, crate::util::min_max::max(&values));
quantity_fn!(standard_deviation, values, {
    let mean_value = statistical::mean(&values);
    statistical::standard_deviation(&values, Some(mean_value))
});

#[test]
fn test_time() {
    let time = Time::new::<millisecond>(123.4);
    assert_eq!(time.value_in(millisecond), 123.4);

    let time_s = time.value_in(second);
    approx::assert_relative_eq!(time_s, 0.1234);

    let time_us = time.value_in(microsecond);
    approx::assert_relative_eq!(time_us, 123400.0);
}

#[test]
fn test_information() {
    let information = Information::new::<kibibyte>(8);
    assert_eq!(information.value_in(byte), 8192);

    let information_kib = information.value_in(kibibyte);
    assert_eq!(information_kib, 8);
}

#[test]
fn test_format() {
    let time = Time::new::<millisecond>(123.4);
    assert_eq!(time.format_auto(Some(TimeUnit::Second)), "0.123 s");
    assert_eq!(time.format_auto(Some(TimeUnit::MilliSecond)), "123.4 ms");
    assert_eq!(time.format_auto(Some(TimeUnit::MicroSecond)), "123400.0 µs");

    let peak_memory_usage = Information::new::<kibibyte>(8);
    assert_eq!(peak_memory_usage.to_string(byte), "8192 B");
    assert_eq!(peak_memory_usage.to_string(kibibyte), "8 KiB");
}

#[test]
fn test_mean() {
    let values = vec![
        Time::new::<millisecond>(123.4),
        Time::new::<millisecond>(234.5),
    ];
    let result = mean(&values);
    assert_eq!(result.format_auto(Some(TimeUnit::MilliSecond)), "178.9 ms");
}

#[test]
fn test_format_duration() {
    let (out_str, out_unit) = Time::new::<second>(1.3).format_auto_with_unit(None);

    assert_eq!("1.300 s", out_str);
    assert_eq!(TimeUnit::Second, out_unit);

    let (out_str, out_unit) = Time::new::<second>(1.0).format_auto_with_unit(None);

    assert_eq!("1.000 s", out_str);
    assert_eq!(TimeUnit::Second, out_unit);

    let (out_str, out_unit) = Time::new::<second>(0.999).format_auto_with_unit(None);

    assert_eq!("999.0 ms", out_str);
    assert_eq!(TimeUnit::MilliSecond, out_unit);

    let (out_str, out_unit) = Time::new::<second>(0.0005).format_auto_with_unit(None);

    assert_eq!("500.0 µs", out_str);
    assert_eq!(TimeUnit::MicroSecond, out_unit);

    let (out_str, out_unit) = Time::new::<second>(0.).format_auto_with_unit(None);

    assert_eq!("0.0 µs", out_str);
    assert_eq!(TimeUnit::MicroSecond, out_unit);

    let (out_str, out_unit) = Time::new::<second>(1000.0).format_auto_with_unit(None);

    assert_eq!("1000.000 s", out_str);
    assert_eq!(TimeUnit::Second, out_unit);
}

#[test]
fn test_format_duration_unit_with_unit() {
    let (out_str, out_unit) =
        Time::new::<second>(1.3).format_auto_with_unit(Some(TimeUnit::Second));

    assert_eq!("1.300 s", out_str);
    assert_eq!(TimeUnit::Second, out_unit);

    let (out_str, out_unit) =
        Time::new::<second>(1.3).format_auto_with_unit(Some(TimeUnit::MilliSecond));

    assert_eq!("1300.0 ms", out_str);
    assert_eq!(TimeUnit::MilliSecond, out_unit);

    let (out_str, out_unit) =
        Time::new::<second>(1.3).format_auto_with_unit(Some(TimeUnit::MicroSecond));

    assert_eq!("1300000.0 µs", out_str);
    assert_eq!(TimeUnit::MicroSecond, out_unit);
}

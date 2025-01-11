use core::f64;
use std::marker::PhantomData;

use serde::ser::SerializeStruct;
use serde::Serializer;
use uom::{si, Conversion};

pub use uom::si::information::{byte, kibibyte};
pub use uom::si::ratio::ratio;
pub use uom::si::time::{hour, microsecond, millisecond, minute, nanosecond, second};

pub use si::f64::{Information, Ratio, Time};

pub use units::TimeUnit;

mod units;

pub trait TimeQuantity {
    fn zero() -> Time {
        Time::new::<second>(0.0)
    }

    fn suitable_unit(&self) -> TimeUnit;

    fn format(&self, u: TimeUnit, precision: Option<usize>) -> String;
    fn format_auto(&self, time_unit: Option<TimeUnit>) -> String;
    fn format_value_auto(&self, time_unit: Option<TimeUnit>) -> String;
}

impl TimeQuantity for Time {
    fn suitable_unit(&self) -> TimeUnit {
        if *self < Time::new::<millisecond>(1.0) {
            TimeUnit::MicroSecond
        } else if *self < Time::new::<second>(1.0) {
            TimeUnit::MilliSecond
        } else {
            TimeUnit::Second
        }
    }

    /// Format the given time duration as a string.
    fn format(&self, u: TimeUnit, precision: Option<usize>) -> String {
        let precision = precision.unwrap_or_else(|| if u == TimeUnit::Second { 3 } else { 1 });
        u.format(*self, precision)
    }

    /// Format the given time duration as a string. The output-unit can be enforced by setting `unit` to
    /// `Some(target_unit)`. If `unit` is `None`, it will be determined automatically.
    fn format_auto(&self, time_unit: Option<TimeUnit>) -> String {
        let time_unit = time_unit.unwrap_or_else(|| self.suitable_unit());

        let value = self.format(time_unit, None);

        format!("{} {}", value, time_unit.short_name())
    }

    /// Like `format_duration`, but returns the target unit as well.
    fn format_value_auto(&self, time_unit: Option<TimeUnit>) -> String {
        let time_unit = time_unit.unwrap_or_else(|| self.suitable_unit());

        self.format(time_unit, None)
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
        Information::new::<byte>(0.)
    }

    fn to_string<U>(&self, u: U) -> String
    where
        U: si::information::Unit + Conversion<f64, T = f64>;
}

impl InformationQuantity for Information {
    fn to_string<U>(&self, u: U) -> String
    where
        U: si::information::Unit + Conversion<f64, T = f64>,
    {
        format!(
            "{}",
            self.into_format_args(u, uom::fmt::DisplayStyle::Abbreviation)
        )
    }
}

pub trait UnsafeRawValue {
    fn unsafe_raw_value(&self) -> f64;
    fn unsafe_from_raw_value(value: f64) -> Self;
}

impl UnsafeRawValue for Time {
    fn unsafe_raw_value(&self) -> f64 {
        self.get::<second>()
    }

    fn unsafe_from_raw_value(value: f64) -> Self {
        Time::new::<second>(value)
    }
}

impl UnsafeRawValue for Information {
    fn unsafe_raw_value(&self) -> f64 {
        self.get::<byte>()
    }

    fn unsafe_from_raw_value(value: f64) -> Self {
        Information::new::<byte>(value)
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
        pub fn $name<Q: UnsafeRawValue>(values: &[Q]) -> Q {
            let $unwrapped_values: Vec<_> = values.iter().map(|q| q.unsafe_raw_value()).collect();
            let result_value = $body;

            Q::unsafe_from_raw_value(result_value)
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

pub fn modified_zscores<Q: UnsafeRawValue>(values: &[Q]) -> Vec<f64> {
    let values: Vec<_> = values.iter().map(|q| q.unsafe_raw_value()).collect();
    crate::outlier_detection::modified_zscores(&values)
}

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
fn test_format() {
    let time = Time::new::<millisecond>(123.4);
    assert_eq!(time.format_auto(Some(TimeUnit::Second)), "0.123 s");
    assert_eq!(time.format_auto(Some(TimeUnit::MilliSecond)), "123.4 ms");
    assert_eq!(time.format_auto(Some(TimeUnit::MicroSecond)), "123400.0 µs");

    let peak_memory_usage = Information::new::<kibibyte>(8.);
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
fn test_suiteable_unit() {
    assert_eq!(Time::new::<second>(1.3).suitable_unit(), TimeUnit::Second);
    assert_eq!(Time::new::<second>(1.0).suitable_unit(), TimeUnit::Second);
    assert_eq!(
        Time::new::<second>(0.999).suitable_unit(),
        TimeUnit::MilliSecond
    );
    assert_eq!(
        Time::new::<second>(0.0005).suitable_unit(),
        TimeUnit::MicroSecond
    );
    assert_eq!(
        Time::new::<second>(0.).suitable_unit(),
        TimeUnit::MicroSecond
    );
    assert_eq!(
        Time::new::<second>(1000.0).suitable_unit(),
        TimeUnit::Second
    );
}

#[test]
fn test_format_duration_unit_with_unit() {
    let out = Time::new::<second>(1.3).format_auto(Some(TimeUnit::Second));
    assert_eq!("1.300 s", out);

    let out = Time::new::<second>(1.3).format_auto(Some(TimeUnit::MilliSecond));
    assert_eq!("1300.0 ms", out);

    let out = Time::new::<second>(1.3).format_auto(Some(TimeUnit::MicroSecond));
    assert_eq!("1300000.0 µs", out);
}

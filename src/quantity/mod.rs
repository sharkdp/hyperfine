use std::marker::PhantomData;

use serde::ser::SerializeStruct;
use serde::Serializer;

use uom::si;

pub use si::f64::{Information, Ratio, Time};
pub use si::information::{byte, gibibyte, kibibyte, mebibyte, tebibyte};
pub use si::ratio::ratio;
pub use si::time::{hour, microsecond, millisecond, minute, nanosecond, second};
pub use uom::num_traits::Zero;

pub mod statistics;

pub const fn const_time_from_seconds(value: f64) -> Time {
    // Quantity::new in uom is not yet const: https://docs.rs/uom/0.36.0/uom/si/struct.Quantity.html
    Time {
        dimension: PhantomData,
        units: PhantomData,
        value,
    }
}

pub fn serialize_time<S>(t: &Time, s: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let mut state = s.serialize_struct("Time", 2)?;
    state.serialize_field("value", &t.get::<second>())?;
    state.serialize_field("unit", "second")?;
    state.end()
}

pub fn serialize_information<S>(i: &Information, s: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let mut state = s.serialize_struct("Information", 2)?;
    state.serialize_field("value", &i.get::<byte>())?;
    state.serialize_field("unit", "byte")?;
    state.end()
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

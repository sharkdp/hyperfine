//! Typed quantities for benchmark measurements.
//!
//! Both time and information use `f64` storage. Whole-byte counts are exact up to 8 PiB.

use std::marker::PhantomData;

pub use uom::num_traits::Zero;
pub use uom::si::f64::{Information, Ratio, Time};
pub use uom::si::information::byte;
pub use uom::si::ratio::ratio;
pub use uom::si::time::{microsecond, nanosecond, second};

pub const fn const_time_from_seconds(value: f64) -> Time {
    // Quantity::new in uom is not yet const: https://docs.rs/uom/0.36.0/uom/si/struct.Quantity.html
    Time {
        dimension: PhantomData,
        units: PhantomData,
        value,
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

//! This module contains common units.

use std::marker::PhantomData;

use crate::quantity::{microsecond, millisecond, second, Time};

/// Supported time units
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TimeUnit {
    Second,
    MilliSecond,
    MicroSecond,
}

struct Dispatcher<U: uom::si::time::Unit + uom::Conversion<f64, T = f64>> {
    u: PhantomData<U>,
}

impl<U: uom::si::time::Unit + uom::Conversion<f64, T = f64>> Dispatcher<U> {
    fn new() -> Self {
        Dispatcher { u: PhantomData }
    }
}

trait UnitImpl {
    fn short_name(&self) -> &'static str;
    fn format_value(&self, value: Time) -> String;
}

impl<U: uom::si::time::Unit + uom::Conversion<f64, T = f64>> UnitImpl for Dispatcher<U> {
    fn short_name(&self) -> &'static str {
        U::abbreviation()
    }

    fn format_value(&self, value: Time) -> String {
        let precision = if U::abbreviation() == "s" { 3 } else { 1 };
        format!("{value:.precision$}", value = value.get::<U>())
    }
}

impl TimeUnit {
    fn dispatch(self) -> Box<dyn UnitImpl> {
        match self {
            TimeUnit::Second => Box::new(Dispatcher::<second>::new()),
            TimeUnit::MilliSecond => Box::new(Dispatcher::<millisecond>::new()),
            TimeUnit::MicroSecond => Box::new(Dispatcher::<microsecond>::new()),
        }
    }

    /// A short abbreviation like `s`, `ms`, or `µs`.
    pub fn short_name(self) -> &'static str {
        self.dispatch().short_name()
    }

    /// Returns the Second value formatted for the Unit.
    pub fn format(self, value: Time) -> String {
        self.dispatch().format_value(value)
    }
}

#[test]
fn test_unit_short_name() {
    assert_eq!("s", TimeUnit::Second.short_name());
    assert_eq!("ms", TimeUnit::MilliSecond.short_name());
    assert_eq!("µs", TimeUnit::MicroSecond.short_name());
}

// Note - the values are rounded when formatted.
#[test]
fn test_unit_format() {
    use crate::quantity::TimeQuantity;

    let value = Time::from_seconds(123.456789);
    assert_eq!("123.457", TimeUnit::Second.format(value));
    assert_eq!("123456.8", TimeUnit::MilliSecond.format(value));

    assert_eq!(
        "1234.6",
        TimeUnit::MicroSecond.format(Time::from_seconds(0.00123456))
    );
}

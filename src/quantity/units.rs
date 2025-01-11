//! This module contains common units.

use std::marker::PhantomData;

use crate::quantity::{hour, microsecond, millisecond, minute, second, Time};

/// Supported time units
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TimeUnit {
    MicroSecond,
    MilliSecond,
    Second,
    Minute,
    Hour,
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
    fn format_value(&self, value: Time, precision: usize) -> String;
}

impl<U: uom::si::time::Unit + uom::Conversion<f64, T = f64>> UnitImpl for Dispatcher<U> {
    fn short_name(&self) -> &'static str {
        U::abbreviation()
    }

    fn format_value(&self, value: Time, precision: usize) -> String {
        format!("{value:.precision$}", value = value.get::<U>())
    }
}

impl TimeUnit {
    fn dispatch(self) -> Box<dyn UnitImpl> {
        match self {
            TimeUnit::MicroSecond => Box::new(Dispatcher::<microsecond>::new()),
            TimeUnit::MilliSecond => Box::new(Dispatcher::<millisecond>::new()),
            TimeUnit::Second => Box::new(Dispatcher::<second>::new()),
            TimeUnit::Minute => Box::new(Dispatcher::<minute>::new()),
            TimeUnit::Hour => Box::new(Dispatcher::<hour>::new()),
        }
    }

    /// A short abbreviation like `s`, `ms`, or `µs`.
    pub fn short_name(self) -> &'static str {
        self.dispatch().short_name()
    }

    /// Formats the quantity as a string in the given Unit.
    pub fn format(self, value: Time, precision: usize) -> String {
        self.dispatch().format_value(value, precision)
    }
}

#[test]
fn test_unit_short_name() {
    assert_eq!("s", TimeUnit::Second.short_name());
    assert_eq!("ms", TimeUnit::MilliSecond.short_name());
    assert_eq!("µs", TimeUnit::MicroSecond.short_name());
    assert_eq!("min", TimeUnit::Minute.short_name());
    assert_eq!("h", TimeUnit::Hour.short_name());
}

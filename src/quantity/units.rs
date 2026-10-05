//! This module contains common units for time and information quantities.

use std::marker::PhantomData;

use crate::quantity::{
    byte, gibibyte, hour, kibibyte, mebibyte, microsecond, millisecond, minute, second, tebibyte,
    Information, Time,
};

pub trait IsUnit {
    type Quantity;

    fn dispatch(&self) -> Box<dyn UnitImpl<Quantity = Self::Quantity>>;

    fn preferred_precision(&self) -> usize;

    fn short_name(&self) -> &'static str {
        self.dispatch().short_name()
    }

    fn format(&self, value: Self::Quantity, precision: usize) -> String {
        self.dispatch().format_value(value, precision)
    }
}

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

impl TimeUnit {
    pub fn preferred_precision(self) -> usize {
        match self {
            TimeUnit::Second => 3,
            _ => 1,
        }
    }
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

/// Supported information units
#[derive(Debug, Clone, Copy, PartialEq, Eq)]

pub enum InformationUnit {
    Byte,
    KibiByte,
    MebiByte,
    GibiByte,
    TebiByte,
}

impl IsUnit for InformationUnit {
    type Quantity = Information;

    fn dispatch(&self) -> Box<dyn UnitImpl<Quantity = Information>> {
        match self {
            InformationUnit::Byte => Box::new(InformationUnitDispatcher::<byte>::new()),
            InformationUnit::KibiByte => Box::new(InformationUnitDispatcher::<kibibyte>::new()),
            InformationUnit::MebiByte => Box::new(InformationUnitDispatcher::<mebibyte>::new()),
            InformationUnit::GibiByte => Box::new(InformationUnitDispatcher::<gibibyte>::new()),
            InformationUnit::TebiByte => Box::new(InformationUnitDispatcher::<tebibyte>::new()),
        }
    }

    fn preferred_precision(&self) -> usize {
        match self {
            InformationUnit::Byte => 0,
            _ => 1,
        }
    }
}

pub trait UnitImpl {
    type Quantity;

    fn short_name(&self) -> &'static str;
    fn format_value(&self, value: Self::Quantity, precision: usize) -> String;
}

struct TimeUnitDispatcher<U: uom::si::time::Unit + uom::Conversion<f64, T = f64>> {
    u: PhantomData<U>,
}

impl<U: uom::si::time::Unit + uom::Conversion<f64, T = f64>> TimeUnitDispatcher<U> {
    fn new() -> Self {
        TimeUnitDispatcher { u: PhantomData }
    }
}

impl<U: uom::si::time::Unit + uom::Conversion<f64, T = f64>> UnitImpl for TimeUnitDispatcher<U> {
    type Quantity = Time;

    fn short_name(&self) -> &'static str {
        U::abbreviation()
    }

    fn format_value(&self, value: Time, precision: usize) -> String {
        format!("{value:.precision$}", value = value.get::<U>())
    }
}

struct InformationUnitDispatcher<U: uom::si::information::Unit + uom::Conversion<f64, T = f64>> {
    u: PhantomData<U>,
}

impl<U: uom::si::information::Unit + uom::Conversion<f64, T = f64>> InformationUnitDispatcher<U> {
    fn new() -> Self {
        InformationUnitDispatcher { u: PhantomData }
    }
}

impl<U: uom::si::information::Unit + uom::Conversion<f64, T = f64>> UnitImpl
    for InformationUnitDispatcher<U>
{
    type Quantity = Information;

    fn short_name(&self) -> &'static str {
        U::abbreviation()
    }

    fn format_value(&self, value: Information, precision: usize) -> String {
        format!("{value:.precision$}", value = value.get::<U>())
    }
}

#[test]
fn test_time_unit_short_name() {
    assert_eq!("s", TimeUnit::Second.short_name());
    assert_eq!("ms", TimeUnit::MilliSecond.short_name());
    assert_eq!("µs", TimeUnit::MicroSecond.short_name());
    assert_eq!("min", TimeUnit::Minute.short_name());
    assert_eq!("h", TimeUnit::Hour.short_name());
}

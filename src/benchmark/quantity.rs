use std::{
    fmt::{self, Display, Formatter},
    ops::{Add, Div, Sub},
};

use serde::{ser::SerializeStruct, Serialize, Serializer};

pub trait IsQuantity {
    type Value;

    const IS_TIME: bool;
    const METRIC_PREFIX_TIME: i32;
    const IS_INFORMATION: bool;
    const BINARY_PREFIX_INFORMATION: i32;

    fn unsafe_from(value: Self::Value) -> Self;
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd)] // TODO: check partialord
pub struct Quantity<
    Value,
    const IS_TIME: bool,
    const METRIC_PREFIX_TIME: i32,
    const IS_INFORMATION: bool,
    const BINARY_PREFIX_INFORMATION: i32,
>(Value);

impl<
        Value,
        const IS_TIME: bool,
        const METRIC_PREFIX_TIME: i32,
        const IS_INFORMATION: bool,
        const BINARY_PREFIX_INFORMATION: i32,
    > IsQuantity
    for Quantity<Value, IS_TIME, METRIC_PREFIX_TIME, IS_INFORMATION, BINARY_PREFIX_INFORMATION>
{
    type Value = Value;
    const IS_TIME: bool = IS_TIME;
    const METRIC_PREFIX_TIME: i32 = METRIC_PREFIX_TIME;
    const IS_INFORMATION: bool = IS_INFORMATION;
    const BINARY_PREFIX_INFORMATION: i32 = BINARY_PREFIX_INFORMATION;

    fn unsafe_from(value: Self::Value) -> Self {
        Self(value)
    }
}

type Time<const PREFIX: i32> = Quantity<f64, true, PREFIX, false, 0>;
type Information<const PREFIX: i32> = Quantity<u64, false, 0, true, PREFIX>;

pub type Second = Time<0>;
pub type MilliSecond = Time<-3>;
pub type MicroSecond = Time<-6>;

pub type Byte = Information<0>;
pub type KibiByte = Information<10>;
pub type MebiByte = Information<20>;
pub type GibiByte = Information<30>;
pub type TebiByte = Information<40>;

impl<
        Value: Copy + Default,
        const IS_TIME: bool,
        const METRIC_PREFIX_TIME: i32,
        const IS_INFORMATION: bool,
        const BINARY_PREFIX_INFORMATION: i32,
    > Quantity<Value, IS_TIME, METRIC_PREFIX_TIME, IS_INFORMATION, BINARY_PREFIX_INFORMATION>
{
    pub const fn new(value: Value) -> Self {
        Self(value)
    }

    pub fn zero() -> Self {
        Self::new(Value::default())
    }

    pub fn value_in<TargetUnit: IsQuantity>(&self) -> Value {
        const {
            assert!(IS_TIME == TargetUnit::IS_TIME);
            assert!(METRIC_PREFIX_TIME == TargetUnit::METRIC_PREFIX_TIME);
            assert!(IS_INFORMATION == TargetUnit::IS_INFORMATION);
            assert!(BINARY_PREFIX_INFORMATION == TargetUnit::BINARY_PREFIX_INFORMATION);
        }

        self.0
    }

    const fn unit_name_long(&self) -> &'static str {
        match (IS_TIME, IS_INFORMATION) {
            (true, false) => match METRIC_PREFIX_TIME {
                -6 => "microsecond",
                -3 => "millisecond",
                0 => "second",
                _ => unreachable!(),
            },
            (false, true) => match BINARY_PREFIX_INFORMATION {
                0 => "byte",
                10 => "kibibyte",
                20 => "mebibyte",
                30 => "gibibyte",
                40 => "tebibyte",
                _ => unreachable!(),
            },
            _ => unreachable!(),
        }
    }
}

impl<const METRIC_PREFIX_TIME: i32> Time<METRIC_PREFIX_TIME> {
    pub fn convert_to<TargetUnit: IsQuantity<Value = f64>>(&self) -> TargetUnit {
        let factor = 10_f64.powi(METRIC_PREFIX_TIME - TargetUnit::METRIC_PREFIX_TIME);

        TargetUnit::unsafe_from(self.0 * factor)
    }
}

impl<const METRIC_PREFIX_TIME: i32> Add<Time<METRIC_PREFIX_TIME>> for Time<METRIC_PREFIX_TIME> {
    type Output = Time<METRIC_PREFIX_TIME>;

    fn add(self, rhs: Time<METRIC_PREFIX_TIME>) -> Self::Output {
        Time::new(self.0 + rhs.0)
    }
}

impl<const METRIC_PREFIX_TIME: i32> Sub<Time<METRIC_PREFIX_TIME>> for Time<METRIC_PREFIX_TIME> {
    type Output = Time<METRIC_PREFIX_TIME>;

    fn sub(self, rhs: Time<METRIC_PREFIX_TIME>) -> Self::Output {
        Time::new(self.0 - rhs.0)
    }
}

impl<const METRIC_PREFIX_TIME: i32> Div<Time<METRIC_PREFIX_TIME>> for Time<METRIC_PREFIX_TIME> {
    type Output = f64;

    fn div(self, rhs: Time<METRIC_PREFIX_TIME>) -> Self::Output {
        self.0 / rhs.0
    }
}

impl<const METRIC_PREFIX_TIME: i32> Display for Time<METRIC_PREFIX_TIME> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let prefix = match METRIC_PREFIX_TIME {
            -6 => "µ",
            -3 => "m",
            0 => "s",
            _ => unreachable!(),
        };
        write!(f, "{} {}s", self.0, prefix)
    }
}

impl<const BINARY_PREFIX_INFORMATION: i32> Display for Information<BINARY_PREFIX_INFORMATION> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let prefix = match BINARY_PREFIX_INFORMATION {
            0 => "",
            10 => "Ki",
            20 => "Mi",
            30 => "Gi",
            40 => "Ti",
            _ => unreachable!(),
        };
        write!(f, "{} {}B", self.0, prefix)
    }
}

impl<
        Value: Copy + Default + Serialize,
        const IS_TIME: bool,
        const METRIC_PREFIX_TIME: i32,
        const IS_INFORMATION: bool,
        const BINARY_PREFIX_INFORMATION: i32,
    > Serialize
    for Quantity<Value, IS_TIME, METRIC_PREFIX_TIME, IS_INFORMATION, BINARY_PREFIX_INFORMATION>
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("Quantity", 2)?;
        state.serialize_field("value", &self.0)?;
        state.serialize_field("unit", self.unit_name_long())?;
        state.end()
    }
}

macro_rules! quantity_fn {
    ($name:ident, $unwrapped_values:ident, $body:expr) => {
        pub fn $name<
            const IS_TIME: bool,
            const METRIC_PREFIX_TIME: i32,
            const IS_INFORMATION: bool,
            const BINARY_PREFIX_INFORMATION: i32,
        >(
            values: &[Quantity<
                f64,
                IS_TIME,
                METRIC_PREFIX_TIME,
                IS_INFORMATION,
                BINARY_PREFIX_INFORMATION,
            >],
        ) -> Quantity<f64, IS_TIME, METRIC_PREFIX_TIME, IS_INFORMATION, BINARY_PREFIX_INFORMATION> {
            let $unwrapped_values: Vec<f64> = values.iter().map(|v| v.0).collect();
            let result_value = $body;
            Quantity::new(result_value)
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
fn test_quantity() {
    let time = MilliSecond::new(123.4);
    assert_eq!(time.value_in::<MilliSecond>(), 123.4);

    let time_s = time.convert_to::<Second>();
    approx::assert_relative_eq!(time_s.value_in::<Second>(), 0.1234);

    let time_us = time.convert_to::<MicroSecond>();
    approx::assert_relative_eq!(time_us.value_in::<MicroSecond>(), 123400.0);
}

#[test]
fn test_format() {
    let time = MilliSecond::new(123.4);
    assert_eq!(time.to_string(), "123.4 ms");

    let peak_memory_usage = KibiByte::new(8);
    assert_eq!(peak_memory_usage.to_string(), "8 KiB");
}

#[test]
fn test_mean() {
    let values = vec![MilliSecond::new(123.4), MilliSecond::new(234.5)];
    let result = mean(&values);
    assert_eq!(result.to_string(), "178.95 ms");
}

//! Metric selection, descriptive statistics, and display units.

use anyhow::{bail, ensure, Result};

use crate::benchmark::measurement::{Measurement, Measurements};
use crate::quantity::statistics::{max, mean, median, min, standard_deviation};
use crate::quantity::{byte, ratio, second, Ratio};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Metric {
    TimeWallClock,
    TimeCpu,
    TimeUser,
    TimeSystem,
    MemoryPeakResident,
    CpuCycles,
    Instructions,
    CacheReferences,
    CacheMisses,
    BranchMisses,
}

impl Metric {
    pub const ALL: [Self; 10] = [
        Self::TimeWallClock,
        Self::TimeCpu,
        Self::TimeUser,
        Self::TimeSystem,
        Self::MemoryPeakResident,
        Self::CpuCycles,
        Self::Instructions,
        Self::CacheReferences,
        Self::CacheMisses,
        Self::BranchMisses,
    ];

    pub fn name(self) -> &'static str {
        match self {
            Self::TimeWallClock => "time_wall_clock",
            Self::TimeCpu => "time_cpu",
            Self::TimeUser => "time_user",
            Self::TimeSystem => "time_system",
            Self::MemoryPeakResident => "memory_peak_resident",
            Self::CpuCycles => "cpu_cycles",
            Self::Instructions => "instructions",
            Self::CacheReferences => "cache_references",
            Self::CacheMisses => "cache_misses",
            Self::BranchMisses => "branch_misses",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::TimeWallClock => "Wall Time",
            Self::TimeCpu => "CPU time",
            Self::TimeUser => "User time",
            Self::TimeSystem => "System time",
            Self::MemoryPeakResident => "Memory",
            Self::CpuCycles => "CPU cycles",
            Self::Instructions => "Instructions",
            Self::CacheReferences => "Cache references",
            Self::CacheMisses => "Cache misses",
            Self::BranchMisses => "Branch misses",
        }
    }

    /// Extract a value in canonical units: seconds, bytes, or counts.
    pub fn value(self, measurement: &Measurement) -> Option<f64> {
        let counters = &measurement.hardware_counters;
        match self {
            Self::TimeWallClock => Some(measurement.time_wall_clock.get::<second>()),
            Self::TimeCpu => Some(measurement.time_cpu.get::<second>()),
            Self::TimeUser => Some(measurement.time_user.get::<second>()),
            Self::TimeSystem => Some(measurement.time_system.get::<second>()),
            Self::MemoryPeakResident => measurement.memory_peak_resident.map(|v| v.get::<byte>()),
            Self::CpuCycles => counters.cpu_cycles.map(|v| v as f64),
            Self::Instructions => counters.instructions.map(|v| v as f64),
            Self::CacheReferences => counters.cache_references.map(|v| v as f64),
            Self::CacheMisses => counters.cache_misses.map(|v| v as f64),
            Self::BranchMisses => counters.branch_misses.map(|v| v as f64),
        }
    }

    pub fn is_time(self) -> bool {
        matches!(
            self,
            Self::TimeWallClock | Self::TimeCpu | Self::TimeUser | Self::TimeSystem
        )
    }

    pub fn is_counter(self) -> bool {
        !self.is_time() && self != Self::MemoryPeakResident
    }

    /// Reject known unsupported configurations before any commands execute.
    /// Availability that depends on permissions or hardware is checked after collection.
    pub fn ensure_available(self, shell: bool) -> Result<()> {
        if self.is_counter() {
            ensure!(
                !shell,
                "Metric '{}' is unavailable when a shell is enabled",
                self.name()
            );
            ensure!(
                cfg!(any(
                    target_os = "macos",
                    all(
                        target_os = "linux",
                        target_pointer_width = "64",
                        any(
                            target_arch = "aarch64",
                            target_arch = "x86_64",
                            target_arch = "riscv64",
                            target_arch = "powerpc64",
                            target_arch = "loongarch64"
                        )
                    )
                )),
                "Metric '{}' is unavailable on this platform",
                self.name()
            );
            ensure!(
                !cfg!(target_os = "macos") || matches!(self, Self::CpuCycles | Self::Instructions),
                "Metric '{}' is unavailable on macOS",
                self.name()
            );
        }
        ensure!(
            !cfg!(windows) || self != Self::MemoryPeakResident,
            "Metric '{}' is unavailable on Windows",
            self.name()
        );
        Ok(())
    }

    pub fn base_unit(self) -> Unit {
        if self.is_time() {
            Unit::new("s", 1.0)
        } else if self.is_counter() {
            Unit::new("count", 1.0)
        } else {
            Unit::new("B", 1.0)
        }
    }

    fn units(self) -> &'static [Unit] {
        if self.is_time() {
            TIME_UNITS
        } else if self.is_counter() {
            COUNT_UNITS
        } else {
            MEMORY_UNITS
        }
    }

    /// Require complete coverage so different metrics describe the same runs.
    pub fn summarize(self, measurements: &Measurements) -> Option<Stats> {
        let values = measurements
            .measurements
            .iter()
            .map(|measurement| self.value(measurement))
            .collect::<Option<Vec<_>>>()?;
        if values.is_empty() {
            return None;
        }
        Some(Stats {
            count: values.len(),
            mean: mean(values.iter().copied()),
            stddev: (values.len() > 1).then(|| {
                standard_deviation(values.iter().map(|&v| Ratio::new::<ratio>(v))).get::<ratio>()
            }),
            median: median(values.iter().copied()),
            min: min(values.iter().copied()),
            max: max(values.iter().copied()),
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MetricSelection {
    pub metric: Metric,
    pub unit: Option<Unit>,
}

impl Default for MetricSelection {
    fn default() -> Self {
        Self {
            metric: Metric::TimeWallClock,
            unit: None,
        }
    }
}

impl MetricSelection {
    /// Parse an ordered, nonempty list of `METRIC[:UNIT]` selections.
    pub fn parse_list(input: &str) -> Result<Vec<Self>> {
        let mut selections = Vec::new();
        for item in input.split(',') {
            let (name, unit_name) = match item.split_once(':') {
                Some((name, unit)) => (name, Some(unit)),
                None => (item, None),
            };
            let metric = match Metric::ALL.iter().copied().find(|m| m.name() == name) {
                Some(metric) => metric,
                None => bail!(
                    "Unknown metric '{name}'. Expected one of: {}",
                    Metric::ALL.map(Metric::name).join(", ")
                ),
            };
            if selections
                .iter()
                .any(|selection: &Self| selection.metric == metric)
            {
                bail!("Metric '{name}' was selected more than once");
            }
            let unit = match unit_name {
                None => None,
                Some(unit_name) => {
                    // Accept an ASCII spelling for microseconds as well.
                    let unit_name = if unit_name == "us" { "µs" } else { unit_name };
                    match metric.units().iter().find(|unit| unit.symbol == unit_name) {
                        Some(unit) => Some(*unit),
                        None => bail!(
                            "Unit '{unit_name}' is not valid for metric '{name}'. Expected one of: {}",
                            metric.units().iter().map(|u| u.symbol).collect::<Vec<_>>().join(", ")
                        ),
                    }
                }
            };
            selections.push(Self { metric, unit });
        }
        Ok(selections)
    }

    pub fn display_unit(self, mean: f64) -> Unit {
        if let Some(unit) = self.unit {
            return unit;
        }
        if mean == 0.0 {
            return self.metric.base_unit();
        }
        let units = if self.metric == Metric::MemoryPeakResident {
            BINARY_MEMORY_UNITS
        } else {
            self.metric.units()
        };
        units
            .iter()
            .rev()
            .find(|unit| mean.abs() >= unit.scale)
            .copied()
            .unwrap_or(units[0])
    }

    /// CSV never automatically scales values.
    pub fn csv_unit(self) -> Unit {
        self.unit.unwrap_or_else(|| self.metric.base_unit())
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Unit {
    pub symbol: &'static str,
    /// The number of canonical units in this unit (e.g. 1e9 for a billion).
    pub scale: f64,
}

impl Unit {
    const fn new(symbol: &'static str, scale: f64) -> Self {
        Self { symbol, scale }
    }

    pub fn format(self, value: f64) -> String {
        let value = self.format_value(value);
        if self.symbol == "count" {
            value
        } else {
            format!("{value} {}", self.symbol)
        }
    }

    pub fn format_value(self, value: f64) -> String {
        let precision = if self.symbol == "s" { 3 } else { 1 };
        format!("{:.precision$}", value / self.scale)
    }
}

const TIME_UNITS: &[Unit] = &[
    Unit::new("ns", 1e-9),
    Unit::new("µs", 1e-6),
    Unit::new("ms", 1e-3),
    Unit::new("s", 1.0),
    Unit::new("min", 60.0),
    Unit::new("h", 3600.0),
];

const BINARY_MEMORY_UNITS: &[Unit] = &[
    Unit::new("B", 1.0),
    Unit::new("KiB", 1024.0),
    Unit::new("MiB", 1_048_576.0),
    Unit::new("GiB", 1_073_741_824.0),
    Unit::new("TiB", 1_099_511_627_776.0),
];

const MEMORY_UNITS: &[Unit] = &[
    Unit::new("B", 1.0),
    Unit::new("kB", 1e3),
    Unit::new("MB", 1e6),
    Unit::new("GB", 1e9),
    Unit::new("TB", 1e12),
    Unit::new("KiB", 1024.0),
    Unit::new("MiB", 1_048_576.0),
    Unit::new("GiB", 1_073_741_824.0),
    Unit::new("TiB", 1_099_511_627_776.0),
];

const COUNT_UNITS: &[Unit] = &[
    Unit::new("count", 1.0),
    Unit::new("k", 1e3),
    Unit::new("M", 1e6),
    Unit::new("B", 1e9),
];

#[derive(Debug, Clone, Copy)]
pub struct Stats {
    pub count: usize,
    pub mean: f64,
    pub stddev: Option<f64>,
    pub median: f64,
    pub min: f64,
    pub max: f64,
}

impl Stats {
    pub fn change_from(&self, reference: &Self) -> Option<f64> {
        (reference.mean != 0.0).then(|| 100.0 * (self.mean / reference.mean - 1.0))
    }

    pub fn format_change_from(&self, reference: &Self) -> String {
        let Some(change) = self.change_from(reference) else {
            return "N/A".to_owned();
        };
        let mut formatted = format!("{change:+.1}%");
        if formatted == "+0.0%" || formatted == "-0.0%" {
            formatted = "0.0%".to_owned();
        }
        formatted
    }

    pub fn format_factor_from(&self, reference: &Self, metric: Metric) -> String {
        if reference.mean > 0.0
            && (self.mean >= 1.3 * reference.mean || self.mean <= 0.7 * reference.mean)
        {
            let decrease = self.mean < reference.mean;
            let factor = if decrease {
                reference.mean / self.mean
            } else {
                self.mean / reference.mean
            };
            let direction = match (metric.is_time(), decrease) {
                (true, true) => "faster",
                (true, false) => "slower",
                (false, true) => "less",
                (false, false) => "more",
            };
            let factor = if factor.is_infinite() {
                "∞".to_owned()
            } else {
                format!("{factor:.1}")
            };
            format!("({factor}x {direction})")
        } else {
            String::new()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::quantity::{Information, Time};

    #[test]
    fn large_changes_include_factors() {
        let reference = Stats {
            count: 1,
            mean: 100.0,
            stddev: None,
            median: 100.0,
            min: 100.0,
            max: 100.0,
        };
        for (mean, metric, expected) in [
            (130.0, Metric::TimeWallClock, "+30.0% (1.3x slower)"),
            (70.0, Metric::TimeCpu, "-30.0% (1.4x faster)"),
            (10.0, Metric::Instructions, "-90.0% (10.0x less)"),
            (1000.0, Metric::MemoryPeakResident, "+900.0% (10.0x more)"),
            (129.99, Metric::TimeWallClock, "+30.0%"),
            (70.01, Metric::Instructions, "-30.0%"),
            (100.0, Metric::Instructions, "0.0%"),
            (0.0, Metric::Instructions, "-100.0% (∞x less)"),
        ] {
            let stats = Stats { mean, ..reference };
            let (percentage, factor) = expected
                .split_once(" (")
                .map_or((expected, String::new()), |(p, f)| (p, format!("({f}")));
            assert_eq!(stats.format_change_from(&reference), percentage);
            assert_eq!(stats.format_factor_from(&reference, metric), factor);
        }
        let zero = Stats {
            mean: 0.0,
            ..reference
        };
        assert_eq!(reference.format_change_from(&zero), "N/A");
    }

    #[test]
    fn cpu_time_statistics_use_per_run_totals() {
        let measurements = Measurements::new(
            [(1.0, 3.0), (3.0, 1.0)]
                .iter()
                .map(|&(user, system)| Measurement {
                    time_cpu: Time::new::<second>(user + system),
                    time_user: Time::new::<second>(user),
                    time_system: Time::new::<second>(system),
                    ..Measurement::default()
                })
                .collect(),
        );
        let selection = MetricSelection::parse_list("time_cpu:ms").unwrap()[0];
        let stats = selection.metric.summarize(&measurements).unwrap();
        assert_eq!(selection.csv_unit().format(stats.mean), "4000.0 ms");
        // User and system time vary, but their per-run total is constant.
        assert_eq!(stats.stddev, Some(0.0));
        assert_eq!(stats.min, 4.0);
        assert_eq!(stats.max, 4.0);
        let json = serde_json::to_value(&measurements).unwrap();
        for sample in json["measurements"].as_array().unwrap() {
            assert_eq!(sample["time_cpu"]["value"], 4.0);
            assert_eq!(sample["time_cpu"]["unit"], "second");
        }
    }

    #[test]
    fn selection_units_and_validation() {
        let selections = MetricSelection::parse_list(
            "instructions:B,memory_peak_resident:B,time_wall_clock:ms,time_user:us",
        )
        .unwrap();
        assert_eq!(selections[0].csv_unit().scale, 1e9);
        assert_eq!(selections[1].csv_unit().scale, 1.0);
        assert_eq!(selections[2].csv_unit().format(0.125), "125.0 ms");
        assert_eq!(selections[3].csv_unit().symbol, "µs");

        let instructions = MetricSelection::parse_list("instructions").unwrap()[0];
        assert_eq!(instructions.csv_unit().symbol, "count");
        assert_eq!(
            instructions.display_unit(143.2e9).format(143.2e9),
            "143.2 B"
        );
        for invalid in [
            "",
            "unknown",
            "time_wall_clock,",
            "instructions,instructions:M",
            "instructions:MiB",
            "memory_peak_resident:ms",
            "time_wall_clock:",
        ] {
            assert!(MetricSelection::parse_list(invalid).is_err(), "{}", invalid);
        }
    }

    #[test]
    fn summaries_require_complete_samples() {
        let mut measurements = Measurements::new(vec![Measurement {
            memory_peak_resident: Some(Information::new::<byte>(10.0)),
            ..Measurement::default()
        }]);
        let single = Metric::MemoryPeakResident.summarize(&measurements).unwrap();
        assert_eq!(single.stddev, None);
        measurements.push(Measurement::default());
        assert!(Metric::MemoryPeakResident
            .summarize(&measurements)
            .is_none());
        measurements.measurements[1].memory_peak_resident = Some(Information::new::<byte>(30.0));
        let stats = Metric::MemoryPeakResident.summarize(&measurements).unwrap();
        assert_eq!(stats.mean, 20.0);
        assert_eq!(stats.change_from(&single), Some(100.0));
        assert_eq!(stats.format_change_from(&single), "+100.0%");
        assert_eq!(single.format_change_from(&stats), "-50.0%");
        assert_eq!(
            Stats {
                mean: 9.999,
                ..single
            }
            .format_change_from(&single),
            "0.0%"
        );
        approx::assert_relative_eq!(stats.stddev.unwrap(), 200.0_f64.sqrt());
        let zero = Metric::TimeWallClock.summarize(&measurements).unwrap();
        assert_eq!(zero.change_from(&zero), None);
        assert_eq!(zero.format_change_from(&zero), "N/A");
    }
}

use std::fmt;

use crate::benchmark::MIN_EXECUTION_TIME;
use crate::metric::{Metric, MetricSelection};
use crate::quantity::millisecond;

pub struct OutlierWarningOptions {
    pub warmup_in_use: bool,
    pub prepare_in_use: bool,
}

/// A list of all possible warnings
pub enum Warnings {
    FastExecutionTime,
    NonZeroExitCode,
    InitialRunOutlier(MetricSelection, f64, OutlierWarningOptions),
}

impl fmt::Display for Warnings {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Warnings::FastExecutionTime => write!(
                f,
                "Command took less than {} ms to complete. Note that the wall-clock measurements might be \
                inaccurate because hyperfine can not calibrate the shell startup time much \
                more precise than this limit. You can try to use the `-N`/`--shell=none` \
                option to disable the shell completely.",
                MIN_EXECUTION_TIME.get::<millisecond>()
            ),
            Warnings::NonZeroExitCode => write!(f, "Ignoring non-zero exit code."),
            Warnings::InitialRunOutlier(selection, value, _) if selection.metric != Metric::TimeWallClock => write!(
                f,
                "The first benchmarking run for this command had an unusually high value for the primary '{}' metric ({}) compared with the rest.",
                selection.metric.name(),
                selection.display_unit(value).format(value),
            ),
            Warnings::InitialRunOutlier(selection, value, ref options) => write!(
                f,
                "The first benchmarking run for this command was significantly slower than the \
                 rest ({time}). This could be caused by (filesystem) caches that were not filled until \
                 after the first run. {hints}",
                time=selection.display_unit(value).format(value),
                hints=match (options.warmup_in_use, options.prepare_in_use) {
                    (true, true) => "You are already using both the '--warmup' option as well \
                    as the '--prepare' option. Consider re-running the benchmark on a quiet system. \
                    Maybe it was a random outlier. Alternatively, consider increasing the warmup \
                    count.",
                    (true, false) => "You are already using the '--warmup' option which helps \
                    to fill these caches before the actual benchmark. You can either try to \
                    increase the warmup count further or re-run this benchmark on a quiet system \
                    in case it was a random outlier. Alternatively, consider using the '--prepare' \
                    option to clear the caches before each timing run.",
                    (false, true) => "You are already using the '--prepare' option which can \
                    be used to clear caches. If you did not use a cache-clearing command with \
                    '--prepare', you can either try that or consider using the '--warmup' option \
                    to fill those caches before the actual benchmark.",
                    (false, false) => "You should consider using the '--warmup' option to fill \
                    those caches before the actual benchmark. Alternatively, use the '--prepare' \
                    option to clear the caches before each timing run."
                }
            ),

        }
    }
}

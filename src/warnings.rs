use std::fmt;

use crate::benchmark::MIN_EXECUTION_TIME;
use crate::format::format_duration;
use crate::units::Second;

/// A list of all possible warnings
pub enum Warnings {
    FastExecutionTime,
    NonZeroExitCode,
    SlowInitialRun(Second),
    OutliersDetected,
}

impl fmt::Display for Warnings {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Warnings::FastExecutionTime => write!(
                f,
                "Command took less than {:.0} ms to complete. Results might be inaccurate.",
                MIN_EXECUTION_TIME * 1e3
            ),
            Warnings::NonZeroExitCode => write!(f, "Ignoring non-zero exit code."),
            Warnings::SlowInitialRun(t_first) => write!(
                f,
                "The first benchmarking run for this command was significantly slower than the \
                 rest ({}). This could be caused by (filesystem) caches that were not filled until \
                 after the first run. You should consider using the '--warmup' option to fill \
                 those caches before the actual benchmark. Alternatively, use the '--prepare' \
                 option to clear the caches before each timing run.",
                format_duration(t_first, None)
            ),
            Warnings::OutliersDetected => write!(
                f,
                "Statistical outliers were detected. Consider re-running this benchmark on a quiet \
                 PC without any interferences from other programs. It might help to use the \
                 '--warmup' or '--prepare' options."
            ),
        }
    }
}

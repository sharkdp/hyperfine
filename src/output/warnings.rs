use std::fmt;

use crate::benchmark::MIN_EXECUTION_TIME;
use crate::output::format::format_duration;
use crate::util::units::Second;

pub struct OutlierWarningOptions {
    pub warmup_in_use: bool,
    pub prepare_in_use: bool,
}

/// Severity level for off-CPU time warning
pub enum OffCpuSeverity {
    /// Ratio 2-3×: minor off-CPU time
    Note,
    /// Ratio 3-5×: substantial off-CPU time
    Warning,
    /// Ratio 5×+: process was mostly off-CPU
    Severe,
}

/// A list of all possible warnings
pub enum Warnings {
    FastExecutionTime,
    NonZeroExitCode,
    SlowInitialRun(Second, OutlierWarningOptions),
    OutliersDetected(OutlierWarningOptions),
    /// Process spent significant time off-CPU (wall time >> user + system time)
    OffCpuTimeDetected {
        wall_time: Second,
        cpu_time: Second,
        ratio: f64,
        severity: OffCpuSeverity,
    },
}

impl fmt::Display for Warnings {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Warnings::FastExecutionTime => write!(
                f,
                "Command took less than {:.0} ms to complete. Note that the results might be \
                inaccurate because hyperfine can not calibrate the shell startup time much \
                more precise than this limit. You can try to use the `-N`/`--shell=none` \
                option to disable the shell completely.",
                MIN_EXECUTION_TIME * 1e3
            ),
            Warnings::NonZeroExitCode => write!(f, "Ignoring non-zero exit code."),
            Warnings::SlowInitialRun(time_first_run, ref options) => write!(
                f,
                "The first benchmarking run for this command was significantly slower than the \
                 rest ({time}). This could be caused by (filesystem) caches that were not filled until \
                 after the first run. {hints}",
                time=format_duration(time_first_run, None),
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
            Warnings::OutliersDetected(ref options) => write!(
                f,
                "Statistical outliers were detected. Consider re-running this benchmark on a quiet \
                 system without any interferences from other programs.{hint}",
                hint=if options.warmup_in_use && options.prepare_in_use {
                    ""
                } else {
                    " It might help to use the '--warmup' or '--prepare' options."
                }
            ),
            Warnings::OffCpuTimeDetected { wall_time, cpu_time, ratio, ref severity } => {
                let wall_str = format_duration(wall_time, None);
                let cpu_str = format_duration(cpu_time, None);

                match severity {
                    OffCpuSeverity::Note => write!(
                        f,
                        "Some off-CPU time detected: mean wall time ({wall_str}) is {ratio:.1}× \
                         the CPU time ({cpu_str}). The benchmark may include scheduling or I/O effects."
                    ),
                    OffCpuSeverity::Warning => write!(
                        f,
                        "Substantial off-CPU time detected: mean wall time ({wall_str}) is {ratio:.1}× \
                         the CPU time ({cpu_str}). The benchmark is partly measuring system-level effects \
                         (scheduling, I/O, memory management) rather than program performance. \
                         Consider investigating with 'perf' or checking for resource contention."
                    ),
                    OffCpuSeverity::Severe => write!(
                        f,
                        "Process was off-CPU for most of its runtime: mean wall time ({wall_str}) is \
                         {ratio:.1}× the CPU time ({cpu_str}). This benchmark is likely not measuring \
                         what you intend. Common causes include: transparent hugepage allocation, \
                         I/O blocking, memory pressure, or CPU throttling. Consider running on a \
                         quiet system or investigating the root cause."
                    ),
                }
            }
        }
    }
}

pub mod benchmark_result;
pub mod relative_speed;
pub mod scheduler;
pub mod timing_result;

use crate::util::units::Second;

/// Threshold for warning about fast execution time
pub const MIN_EXECUTION_TIME: Second = 5e-3;

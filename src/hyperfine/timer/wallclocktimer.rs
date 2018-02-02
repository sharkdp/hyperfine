use std::time::Instant;

use hyperfine::internal::Second;
use hyperfine::timer::Timer;

pub struct WallClockTimer {
    start: Instant,
}

impl Timer for WallClockTimer {
    type Result = Second;

    fn start() -> WallClockTimer {
        WallClockTimer {
            start: Instant::now(),
        }
    }

    fn stop(&self) -> Second {
        let duration = self.start.elapsed();
        duration.as_secs() as f64 + (duration.subsec_nanos() as f64) * 1e-9
    }
}

use std::time::Instant;

use crate::util::units::Second;

pub struct WallClockTimer {
    start: Instant,
}

impl WallClockTimer {
    pub fn start() -> WallClockTimer {
        WallClockTimer {
            start: Instant::now(),
        }
    }

    pub fn stop(&self) -> Second {
        let duration = self.start.elapsed();
        duration.as_secs() as f64 + f64::from(duration.subsec_nanos()) * 1e-9
    }
}

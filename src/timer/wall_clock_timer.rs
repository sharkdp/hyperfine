use std::time::Instant;

use crate::benchmark::quantity::Second;

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
        // TODO
        Second::new(duration.as_secs() as f64 + f64::from(duration.subsec_nanos()) * 1e-9)
    }
}

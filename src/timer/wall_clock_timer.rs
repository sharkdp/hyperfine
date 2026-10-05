use std::time::Instant;

use crate::quantity::{Time, TimeQuantity};

pub struct WallClockTimer {
    start: Instant,
}

impl WallClockTimer {
    pub fn start() -> WallClockTimer {
        WallClockTimer {
            start: Instant::now(),
        }
    }

    pub fn stop(&self) -> Time {
        let duration = self.start.elapsed();

        Time::from_seconds(duration.as_secs() as f64)
            + Time::from_nanoseconds(duration.subsec_nanos() as f64)
    }
}

use std::time::Instant;

use crate::quantity::{nanosecond, second, Time};

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

        Time::new::<second>(duration.as_secs() as f64)
            + Time::new::<nanosecond>(duration.subsec_nanos() as f64)
    }
}

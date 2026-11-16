use indicatif::{FormattedDuration, ProgressBar, ProgressState, ProgressStyle};
use std::fmt;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use crate::options::OutputStyleOption;

#[cfg(not(windows))]
const TICK_SETTINGS: (&str, u64) = ("⠋⠙⠹⠸⠼⠴⠦⠧⠇⠏ ", 80);

#[cfg(windows)]
const TICK_SETTINGS: (&str, u64) = (r"+-x| ", 200);

/// Number of completed steps that contribute to the rolling average.
///
/// This is the same window that indicatif uses internally.
const ETA_SAMPLE_COUNT: usize = 16;

// Keep the message area 30 columns wide across preparation, the initial run,
// and estimates so that switching between them does not move the bar.
const DEFAULT_TEMPLATE: &str = " {spinner} {msg:<30} {wide_bar} ETA {eta_precise} ";
const INITIAL_TEMPLATE: &str =
    " {spinner} Initial run: {initial_elapsed:<17} {wide_bar} ETA {eta_precise} ";

/// Estimates the remaining time from the durations of completed steps.
///
/// indicatif's built-in estimator only records data when the position changes,
/// so the ETA stays frozen while a single, long-running benchmark step is in
/// progress. This estimator additionally subtracts the time that has already
/// been spent on the current step, allowing the ETA to decrease continuously.
#[derive(Debug)]
struct EtaEstimator {
    position: u64,
    step_started: Instant,
    step_durations: [Duration; ETA_SAMPLE_COUNT],
    sample_count: usize,
    next_sample: usize,
}

impl EtaEstimator {
    fn new(now: Instant) -> Self {
        Self {
            position: 0,
            step_started: now,
            step_durations: [Duration::ZERO; ETA_SAMPLE_COUNT],
            sample_count: 0,
            next_sample: 0,
        }
    }

    fn reset(&mut self, position: u64, now: Instant) {
        self.position = position;
        self.step_started = now;
        self.sample_count = 0;
        self.next_sample = 0;
    }

    /// Record the current progress and return the estimated remaining time.
    ///
    /// Returns `None` if no step has completed yet, in which case the estimate
    /// of the progress bar itself should be used instead.
    fn observe(&mut self, position: u64, length: Option<u64>, now: Instant) -> Option<Duration> {
        if position < self.position {
            self.reset(position, now);
        }

        let completed_steps = position - self.position;
        if completed_steps > 0 {
            let elapsed = now.saturating_duration_since(self.step_started);
            self.step_durations[self.next_sample] = elapsed / completed_steps as u32;
            self.next_sample = (self.next_sample + 1) % ETA_SAMPLE_COUNT;
            self.sample_count = (self.sample_count + 1).min(ETA_SAMPLE_COUNT);
            self.position = position;
            self.step_started = now;
        }

        let length = length?;
        if position >= length || self.sample_count == 0 {
            return None;
        }

        let total: Duration = self.step_durations[..self.sample_count].iter().sum();
        let average = total / self.sample_count as u32;
        let remaining_steps = (length - position).min(u64::from(u32::MAX)) as u32;
        let remaining_after_current = average
            .checked_mul(remaining_steps.saturating_sub(1))
            .unwrap_or(Duration::MAX);
        let in_flight = now.saturating_duration_since(self.step_started);
        let remaining_current = average.saturating_sub(in_flight);

        Some(remaining_current + remaining_after_current)
    }
}

/// Format `{eta_precise}` with [`EtaEstimator`] so that the ETA is refreshed
/// during long steps as well.
fn eta_precise_formatter(
) -> impl Fn(&ProgressState, &mut dyn fmt::Write) + Send + Sync + Clone + 'static {
    let estimator = Arc::new(Mutex::new(EtaEstimator::new(Instant::now())));
    move |state: &ProgressState, w: &mut dyn fmt::Write| {
        let remaining = estimator
            .lock()
            .expect("ETA estimator lock poisoned")
            .observe(state.pos(), state.len(), Instant::now())
            .unwrap_or_else(|| state.eta());
        write!(w, "{}", FormattedDuration(remaining)).unwrap();
    }
}

fn style_with_template(bar: &ProgressBar, template: &str) -> ProgressStyle {
    let template = if bar.prefix().is_empty() {
        template.to_owned()
    } else {
        format!("{{prefix}}\n{template}")
    };
    bar.style().template(&template).expect("no template error")
}

/// Show the benchmark heading above the progress bar, clearing both on completion.
pub fn set_benchmark_header(bar: &ProgressBar, header: String) {
    bar.set_prefix(header);
    bar.set_style(style_with_template(bar, DEFAULT_TEMPLATE));
}

/// Show the time elapsed since the initial benchmark command started.
pub fn start_initial_measurement(bar: &ProgressBar, started: Instant) {
    let style = style_with_template(bar, INITIAL_TEMPLATE).with_key(
        "initial_elapsed",
        move |_: &ProgressState, w: &mut dyn fmt::Write| {
            write!(w, "{}", FormattedDuration(started.elapsed())).unwrap();
        },
    );
    bar.set_style(style);
}

/// Stop showing the initial timer and display the next phase's message.
pub fn finish_initial_measurement(bar: &ProgressBar, message: String) {
    bar.set_message(message);
    bar.set_style(style_with_template(bar, DEFAULT_TEMPLATE));
}

/// Return a pre-configured progress bar
pub fn get_progress_bar(length: u64, msg: &str, option: OutputStyleOption) -> ProgressBar {
    let progressbar_style = match option {
        OutputStyleOption::Basic | OutputStyleOption::Color => ProgressStyle::default_bar(),
        _ => ProgressStyle::default_spinner()
            .tick_chars(TICK_SETTINGS.0)
            .template(DEFAULT_TEMPLATE)
            .expect("no template error")
            .with_key("eta_precise", eta_precise_formatter()),
    };

    let progress_bar = match option {
        OutputStyleOption::Basic | OutputStyleOption::Color => ProgressBar::hidden(),
        _ => ProgressBar::new(length),
    };
    progress_bar.set_style(progressbar_style);
    progress_bar.enable_steady_tick(Duration::from_millis(TICK_SETTINGS.1));
    progress_bar.set_message(msg.to_owned());

    progress_bar
}

#[cfg(test)]
mod tests {
    use super::*;

    fn at(base: Instant, seconds: u64) -> Instant {
        base + Duration::from_secs(seconds)
    }

    #[test]
    fn no_estimate_without_completed_steps() {
        let base = Instant::now();
        let mut estimator = EtaEstimator::new(base);

        assert_eq!(estimator.observe(0, Some(3), at(base, 1)), None);
    }

    #[test]
    fn remaining_time_decreases_while_a_step_is_running() {
        let base = Instant::now();
        let mut estimator = EtaEstimator::new(base);

        assert_eq!(
            estimator.observe(1, Some(3), at(base, 10)),
            Some(Duration::from_secs(20))
        );
        assert_eq!(
            estimator.observe(1, Some(3), at(base, 12)),
            Some(Duration::from_secs(18))
        );
        assert_eq!(
            estimator.observe(1, Some(3), at(base, 20)),
            Some(Duration::from_secs(10))
        );
    }

    #[test]
    fn remaining_time_does_not_underflow_for_overdue_steps() {
        let base = Instant::now();
        let mut estimator = EtaEstimator::new(base);

        estimator.observe(1, Some(3), at(base, 10));
        assert_eq!(
            estimator.observe(1, Some(3), at(base, 60)),
            Some(Duration::from_secs(10))
        );
    }

    #[test]
    fn remaining_time_uses_average_of_completed_steps() {
        let base = Instant::now();
        let mut estimator = EtaEstimator::new(base);

        estimator.observe(1, Some(4), at(base, 4));
        assert_eq!(
            estimator.observe(2, Some(4), at(base, 12)),
            Some(Duration::from_secs(12))
        );
    }

    #[test]
    fn no_estimate_once_all_steps_are_done() {
        let base = Instant::now();
        let mut estimator = EtaEstimator::new(base);

        estimator.observe(1, Some(2), at(base, 5));
        assert_eq!(estimator.observe(2, Some(2), at(base, 10)), None);
    }

    #[test]
    fn backwards_jumps_reset_the_estimate() {
        let base = Instant::now();
        let mut estimator = EtaEstimator::new(base);

        estimator.observe(2, Some(4), at(base, 10));
        assert_eq!(estimator.observe(0, Some(4), at(base, 11)), None);
        assert_eq!(
            estimator.observe(1, Some(4), at(base, 13)),
            Some(Duration::from_secs(6))
        );
    }
}

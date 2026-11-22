use indicatif::{FormattedDuration, ProgressBar, ProgressState, ProgressStyle};
use std::fmt;
use std::sync::atomic::AtomicU64;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use crate::options::OutputStyleOption;

#[cfg(not(windows))]
const TICK_SETTINGS: (&str, u64) = ("⠋⠙⠹⠸⠼⠴⠦⠧⠇⠏ ", 80);

#[cfg(windows)]
const TICK_SETTINGS: (&str, u64) = (r"+-x| ", 200);

// Keep the message area 30 columns wide across preparation, the initial run,
// and estimates so that switching between them does not move the bar.
const DEFAULT_TEMPLATE: &str = " {spinner} {msg:<30} {wide_bar} ETA {remaining} ";
const INITIAL_TEMPLATE: &str =
    " {spinner} Initial run: {initial_elapsed:<17} {wide_bar} ETA {remaining} ";

/// The largest ETA that is shown, about 5 million years. indicatif converts the ETA to
/// nanoseconds, which cannot hold more than 584 years.
const MAX_ETA: Duration = Duration::from_secs(1 << 47);

/// The averaged time of a run is kept in whole units of this many nanoseconds, so that the
/// rounding of the average does not show up as fractions of a second in the ETA.
const AVERAGE_UNIT_NANOS: u128 = 10_000_000;

fn style_with_template(bar: &ProgressBar, template: &str) -> ProgressStyle {
    let template = if bar.prefix().is_empty() {
        template.to_owned()
    } else {
        format!("{{prefix}}\n{template}")
    };
    bar.style().template(&template).expect("no template error")
}

fn nanos(duration: Duration) -> u128 {
    duration.as_secs() as u128 * 1_000_000_000 + duration.subsec_nanos() as u128
}

fn duration_from_nanos(nanos: u128) -> Duration {
    Duration::new(
        (nanos / 1_000_000_000) as u64,
        (nanos % 1_000_000_000) as u32,
    )
}

/// The time that every finished run of the bar took, in the order they finished. It starts with
/// the initial run, which hyperfine runs before the bar and which counts as a step like the
/// runs of the bar.
#[derive(Default)]
struct Runs {
    finished: u64,
    times: Vec<Duration>,
}

impl Runs {
    /// The time of a finished run, rounded to whole units. The time of a run is the time that
    /// passed since the run before it finished, and the initial run is also one of them because
    /// it is part of the steps that indicatif counts.
    fn average(&self) -> Duration {
        if self.times.is_empty() {
            return Duration::ZERO;
        }
        let total = self.total_nanos();
        duration_from_nanos(
            total / self.times.len() as u128 / AVERAGE_UNIT_NANOS * AVERAGE_UNIT_NANOS,
        )
    }

    fn run_finished(&mut self, state: &ProgressState) {
        let elapsed = nanos(state.elapsed());
        self.times.push(duration_from_nanos(
            elapsed.saturating_sub(self.total_nanos()),
        ));
        self.finished = state.pos();
    }

    /// The time from the start of the bar to the end of the last run that finished
    fn total_nanos(&self) -> u128 {
        self.times.iter().map(|run| nanos(*run)).sum()
    }
}

/// Write the ETA of the remaining runs, less the time that the run in progress has taken. The
/// countdown is the time of the run in progress, because indicatif starts counting the remaining
/// runs at the line that is running, or at the line before it once the initial run is done.
fn format_remaining(
    state: &ProgressState,
    w: &mut dyn fmt::Write,
    runs: &mut Runs,
    run: Duration,
) -> Duration {
    if runs.finished != state.pos() {
        runs.run_finished(state);
    }

    let left = if state.is_finished() || state.len().is_none() {
        Duration::ZERO
    } else if runs.finished == 0 {
        // Nothing is known about the runs yet, so the countdown starts with this run
        run
    } else {
        let remaining_runs = state.len().unwrap_or(0).saturating_sub(runs.finished);
        let remaining =
            duration_from_nanos(nanos(runs.average()).saturating_mul(remaining_runs as u128));
        // Before the countdown of the remaining runs is set up, the countdown holds the time
        // since the bar started instead, and would count a line that has not started yet.
        let counted = nanos(state.eta()).min(nanos(state.elapsed()));
        remaining.saturating_sub(run.saturating_sub(duration_from_nanos(counted)))
    };

    write!(w, "{}", FormattedDuration(left)).unwrap();
    left
}

/// Replace indicatif's ETA with one that counts the time of the run in progress.
///
/// indicatif computes its ETA as seconds per step multiplied by the steps that are left, and a
/// step of a hyperfine bar is a whole benchmark run. That estimate therefore only moves when a
/// run starts or ends. During a run that takes hours it stays where it was, and at the start of
/// the next run it grows, because the run that took hours counts towards the average. The ETA
/// here predicts what the remaining runs will take and subtracts what the run in progress has
/// taken so far, so it goes down while the bar redraws through that run.
fn apply_eta<F>(bar: &ProgressBar, template: &str, current_run: F)
where
    F: Fn() -> Duration + Send + Sync + 'static,
{
    let current_run = Arc::new(current_run);
    let runs = Arc::new(Mutex::new(Runs::default()));
    let style = style_with_template(bar, template).with_key(
        "remaining",
        move |state: &ProgressState, w: &mut dyn fmt::Write| {
            let run = current_run().min(MAX_ETA);
            let mut runs = runs.lock().unwrap();
            format_remaining(state, w, &mut runs, run);
        },
    );
    bar.set_style(style);
}

/// Show the benchmark heading above the progress bar, clearing both on completion.
pub fn set_benchmark_header(bar: &ProgressBar, header: String) {
    bar.set_prefix(header);
    apply_eta(bar, DEFAULT_TEMPLATE, || Duration::ZERO);
}

/// Show the time elapsed since the initial benchmark command started.
/// `current_run` holds the time that the running benchmark line has taken.
pub fn start_initial_measurement(bar: &ProgressBar, started: Instant, current_run: Arc<AtomicU64>) {
    let style = style_with_template(bar, INITIAL_TEMPLATE).with_key(
        "initial_elapsed",
        move |_: &ProgressState, w: &mut dyn fmt::Write| {
            write!(w, "{}", FormattedDuration(started.elapsed())).unwrap();
        },
    );
    bar.set_style(style);
    apply_eta(bar, INITIAL_TEMPLATE, move || {
        Duration::from_millis(current_run.load(std::sync::atomic::Ordering::Relaxed))
    });
}

/// Stop showing the initial timer and display the next phase's message.
pub fn finish_initial_measurement(bar: &ProgressBar, message: String) {
    bar.set_message(message);
    apply_eta(bar, DEFAULT_TEMPLATE, || Duration::ZERO);
}

/// Return a pre-configured progress bar
pub fn get_progress_bar(length: u64, msg: &str, option: OutputStyleOption) -> ProgressBar {
    let progress_bar = match option {
        OutputStyleOption::Basic | OutputStyleOption::Color => ProgressBar::hidden(),
        _ => ProgressBar::new(length),
    };
    match option {
        OutputStyleOption::Basic | OutputStyleOption::Color => {
            progress_bar.set_style(ProgressStyle::default_bar())
        }
        _ => apply_eta(&progress_bar, DEFAULT_TEMPLATE, || Duration::ZERO),
    }
    progress_bar.enable_steady_tick(Duration::from_millis(TICK_SETTINGS.1));
    progress_bar.set_message(msg.to_owned());

    progress_bar
}

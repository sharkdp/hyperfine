use indicatif::{FormattedDuration, ProgressBar, ProgressState, ProgressStyle};
use std::fmt;
use std::time::{Duration, Instant};

use crate::options::OutputStyleOption;

#[cfg(not(windows))]
const TICK_SETTINGS: (&str, u64) = ("⠋⠙⠹⠸⠼⠴⠦⠧⠇⠏ ", 80);

#[cfg(windows)]
const TICK_SETTINGS: (&str, u64) = (r"+-x| ", 200);

// Keep the message area 30 columns wide across preparation, the initial run,
// and estimates so that switching between them does not move the bar.
const DEFAULT_TEMPLATE: &str = " {spinner} {msg:<30} {wide_bar} ETA {eta_precise} ";
const INITIAL_TEMPLATE: &str =
    " {spinner} Initial run: {initial_elapsed:<17} {wide_bar} ETA {eta_precise} ";

/// Show the time elapsed since the initial benchmark command started.
pub fn start_initial_measurement(bar: &ProgressBar, started: Instant) {
    let style = bar
        .style()
        .template(INITIAL_TEMPLATE)
        .expect("no template error")
        .with_key(
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
    bar.set_style(
        bar.style()
            .template(DEFAULT_TEMPLATE)
            .expect("no template error"),
    );
}

/// Return a pre-configured progress bar
pub fn get_progress_bar(length: u64, msg: &str, option: OutputStyleOption) -> ProgressBar {
    let progressbar_style = match option {
        OutputStyleOption::Basic | OutputStyleOption::Color => ProgressStyle::default_bar(),
        _ => ProgressStyle::default_spinner()
            .tick_chars(TICK_SETTINGS.0)
            .template(DEFAULT_TEMPLATE)
            .expect("no template error"),
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

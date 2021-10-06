use indicatif::{ProgressBar, ProgressStyle};

use crate::options::OutputStyleOption;

#[cfg(not(windows))]
const TICK_SETTINGS: (&str, u64) = ("⠋⠙⠹⠸⠼⠴⠦⠧⠇⠏ ", 80);

#[cfg(windows)]
const TICK_SETTINGS: (&str, u64) = (r"+-x| ", 200);

/// Return a pre-configured progress bar
pub fn get_progress_bar(
    length: u64,
    msg: &str,
    option: OutputStyleOption,
    show_duration: bool,
) -> ProgressBar {
    let progress_bar_template = if show_duration {
        " {spinner} {msg:<30} {wide_bar}  AT {duration_precise}"
    } else {
        " {spinner} {msg:<30} {wide_bar} ETA {eta_precise}"
    };

    let progressbar_style = match option {
        OutputStyleOption::Basic | OutputStyleOption::Color => ProgressStyle::default_bar(),
        _ => ProgressStyle::default_spinner()
            .tick_chars(TICK_SETTINGS.0)
            .template(progress_bar_template),
    };

    let progress_bar = match option {
        OutputStyleOption::Basic | OutputStyleOption::Color => ProgressBar::hidden(),
        _ => ProgressBar::new(length),
    };
    progress_bar.set_style(progressbar_style);
    progress_bar.enable_steady_tick(TICK_SETTINGS.1);
    progress_bar.set_message(msg.to_owned());

    progress_bar
}

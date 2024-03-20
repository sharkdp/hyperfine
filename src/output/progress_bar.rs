use indicatif::{ProgressBar, ProgressStyle};
use std::time::Duration;

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
    show_elapsed: bool,
) -> ProgressBar {
    let template_str = match show_elapsed {
        true => " {spinner} {msg:<30} {wide_bar} ET {elapsed_precise} ETA {eta_precise} ",
        false => " {spinner} {msg:<30} {wide_bar} ETA {eta_precise} ",
    };

    let progressbar_style = match option {
        OutputStyleOption::Basic | OutputStyleOption::Color => ProgressStyle::default_bar(),
        _ => ProgressStyle::default_spinner()
            .tick_chars(TICK_SETTINGS.0)
            .template(template_str)
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

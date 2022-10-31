use indicatif::{ProgressBar, ProgressStyle};
use std::time::Duration;

use crate::options::OutputStyleOption;

#[cfg(not(windows))]
const TICK_SETTINGS: (&str, u64) = ("⠋⠙⠹⠸⠼⠴⠦⠧⠇⠏ ", 80);

#[cfg(windows)]
const TICK_SETTINGS: (&str, u64) = (r"+-x| ", 200);

const DEFAULT_MESSAGE_TEMPLATE: &str = "{msg:<30}";

fn create_progress_template(msg_template: &str) -> String {
    format!(
        " {{spinner}} {} {{wide_bar}} ETA {{eta_precise}}",
        msg_template
    )
}

/// Replace the usual `message` in a progress bar with the result of evaluating `template`.
/// The `template` may contain a `ProgressBar`'s templated fields.
#[must_use]
pub fn replace_message_template(bar: ProgressBar, template: &str) -> ProgressBar {
    let old_style = bar.style();
    let new_template = create_progress_template(template);
    let new_style = old_style
        .template(&new_template)
        .expect("no template error");
    bar.with_style(new_style)
}

/// Reset a progress bar's template to the default.
/// This is useful after calling `replace_message_template()`.
#[must_use]
pub fn reset_progress_template(bar: ProgressBar) -> ProgressBar {
    replace_message_template(bar, DEFAULT_MESSAGE_TEMPLATE)
}

/// Return a pre-configured progress bar
pub fn get_progress_bar(length: u64, msg: &str, option: OutputStyleOption) -> ProgressBar {
    let template = create_progress_template(DEFAULT_MESSAGE_TEMPLATE);
    let progressbar_style = match option {
        OutputStyleOption::Basic | OutputStyleOption::Color => ProgressStyle::default_bar(),
        _ => ProgressStyle::default_spinner()
            .tick_chars(TICK_SETTINGS.0)
            .template(&template)
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

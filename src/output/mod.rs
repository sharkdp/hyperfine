pub mod progress_bar;
pub mod warnings;

/// Mark console write errors so broken pipes can be handled separately from file errors.
macro_rules! console_writeln {
    ($writer:expr $(, $($args:tt)*)?) => {
        writeln!($writer $(, $($args)*)?)
            .map_err($crate::error::ConsoleOutputError)
    };
}

pub(crate) use console_writeln;

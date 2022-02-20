use std::process::ExitStatus;

#[cfg(unix)]
pub fn extract_exit_code(status: ExitStatus) -> Option<i32> {
    use std::os::unix::process::ExitStatusExt;

    // From the ExitStatus::code documentation:
    //
    //   "On Unix, this will return None if the process was terminated by a signal."
    //
    // In that case, ExitStatusExt::signal should never return None.
    //
    // To differentiate between "normal" exit codes and signals, we are using a technique
    // similar to bash (https://tldp.org/LDP/abs/html/exitcodes.html) and add 128 to the
    // signal value.
    status.code().or_else(|| status.signal().map(|s| s + 128))
}

#[cfg(not(unix))]
pub fn extract_exit_code(status: ExitStatus) -> Option<i32> {
    status.code()
}

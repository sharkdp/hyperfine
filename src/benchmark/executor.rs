use std::process::{ExitStatus, Stdio};

use crate::command::Command;
use crate::execute::execute_and_measure;
use crate::options::{CmdFailureAction, CommandOutputPolicy, Options, OutputStyleOption, Shell};
use crate::output::progress_bar::get_progress_bar;
use crate::timer::wallclocktimer::WallClockTimer;
use crate::util::randomized_environment_offset;
use crate::util::units::Second;

use super::timing_result::TimingResult;

use anyhow::{bail, Result};
use statistical::mean;

pub trait Executor {
    /// Run the given command and measure the execution time
    fn run_command_and_measure(
        &self,
        command: &Command<'_>,
        command_failure_action: Option<CmdFailureAction>,
    ) -> Result<(TimingResult, ExitStatus)>;

    /// Perform a calibration of this executor. For example,
    /// when running commands through a shell, we need to
    /// measure the shell spawning time separately in order
    /// to subtract it from the full runtime later.
    fn calibrate(&mut self) -> Result<()>;

    /// Return the time overhead for this executor when
    /// performing a measurement. This should return the time
    /// that is being used in addition to the actual runtime
    /// of the command.
    fn time_overhead(&self) -> Second;
}

pub struct ShellExecutor<'a> {
    options: &'a Options,
    shell: &'a Shell,
    shell_spawning_time: Option<TimingResult>,
}

impl<'a> ShellExecutor<'a> {
    fn subtract_shell_spawning_time(&self, time: Second, shell_spawning_time: Second) -> Second {
        if time < shell_spawning_time {
            0.0
        } else {
            time - shell_spawning_time
        }
    }

    pub fn new(shell: &'a Shell, options: &'a Options) -> Self {
        ShellExecutor {
            shell,
            options,
            shell_spawning_time: None,
        }
    }
}

impl<'a> Executor for ShellExecutor<'a> {
    fn run_command_and_measure(
        &self,
        command: &Command<'_>,
        command_failure_action: Option<CmdFailureAction>,
    ) -> Result<(TimingResult, ExitStatus)> {
        let (stdout, stderr) = match self.options.command_output_policy {
            CommandOutputPolicy::Discard => (Stdio::null(), Stdio::null()),
            CommandOutputPolicy::Forward => (Stdio::inherit(), Stdio::inherit()),
        };

        let mut command_builder = self.shell.command();
        command_builder
            .arg(if cfg!(windows) { "/C" } else { "-c" })
            .arg(command.get_command_line())
            .env(
                "HYPERFINE_RANDOMIZED_ENVIRONMENT_OFFSET",
                randomized_environment_offset::value(),
            )
            .stdin(Stdio::null())
            .stdout(stdout)
            .stderr(stderr);
        let error_message = format!("Failed to run command '{}'", command.get_command_line());

        let wallclock_timer = WallClockTimer::start();
        let result = execute_and_measure(command_builder, &error_message)?;
        let mut time_real = wallclock_timer.stop();

        let mut time_user = result.user_time;
        let mut time_system = result.system_time;

        if command_failure_action.unwrap_or(self.options.command_failure_action)
            == CmdFailureAction::RaiseError
            && !result.status.success()
        {
            bail!(
                "{}. Use the '-i'/'--ignore-failure' option if you want to ignore this. \
                Alternatively, use the '--show-output' option to debug what went wrong.",
                result.status.code().map_or(
                    "The process has been terminated by a signal".into(),
                    |c| format!("Command terminated with non-zero exit code: {}", c)
                )
            );
        }

        // Subtract shell spawning time
        if let Some(spawning_time) = self.shell_spawning_time {
            time_real = self.subtract_shell_spawning_time(time_real, spawning_time.time_real);
            time_user = self.subtract_shell_spawning_time(time_user, spawning_time.time_user);
            time_system = self.subtract_shell_spawning_time(time_system, spawning_time.time_system);
        }

        Ok((
            TimingResult {
                time_real,
                time_user,
                time_system,
            },
            result.status,
        ))
    }

    /// Measure the average shell spawning time
    fn calibrate(&mut self) -> Result<()> {
        const COUNT: u64 = 50;
        let progress_bar = if self.options.output_style != OutputStyleOption::Disabled {
            Some(get_progress_bar(
                COUNT,
                "Measuring shell spawning time",
                self.options.output_style,
            ))
        } else {
            None
        };

        let mut times_real: Vec<Second> = vec![];
        let mut times_user: Vec<Second> = vec![];
        let mut times_system: Vec<Second> = vec![];

        for _ in 0..COUNT {
            // Just run the shell without any command
            let res = self.run_command_and_measure(&Command::new(None, ""), None);

            match res {
                Err(_) => {
                    let shell_cmd = if cfg!(windows) {
                        format!("{} /C \"\"", self.shell)
                    } else {
                        format!("{} -c \"\"", self.shell)
                    };

                    bail!(
                        "Could not measure shell execution time. Make sure you can run '{}'.",
                        shell_cmd
                    );
                }
                Ok((r, _)) => {
                    times_real.push(r.time_real);
                    times_user.push(r.time_user);
                    times_system.push(r.time_system);
                }
            }

            if let Some(bar) = progress_bar.as_ref() {
                bar.inc(1)
            }
        }

        if let Some(bar) = progress_bar.as_ref() {
            bar.finish_and_clear()
        }

        self.shell_spawning_time = Some(TimingResult {
            time_real: mean(&times_real),
            time_user: mean(&times_user),
            time_system: mean(&times_system),
        });

        Ok(())
    }

    fn time_overhead(&self) -> Second {
        self.shell_spawning_time.unwrap().time_real
    }
}

#[derive(Clone)]
pub struct MockExecutor {
    shell: Option<String>,
}

impl MockExecutor {
    pub fn new(shell: Option<String>) -> Self {
        MockExecutor { shell }
    }

    fn extract_time<S: AsRef<str>>(sleep_command: S) -> Second {
        assert!(sleep_command.as_ref().starts_with("sleep "));
        sleep_command
            .as_ref()
            .trim_start_matches("sleep ")
            .parse::<Second>()
            .unwrap()
    }
}

impl Executor for MockExecutor {
    fn run_command_and_measure(
        &self,
        command: &Command<'_>,
        _command_failure_action: Option<CmdFailureAction>,
    ) -> Result<(TimingResult, ExitStatus)> {
        #[cfg(unix)]
        let status = {
            use std::os::unix::process::ExitStatusExt;
            ExitStatus::from_raw(0)
        };

        #[cfg(windows)]
        let status = {
            use std::os::windows::process::ExitStatusExt;
            ExitStatus::from_raw(0)
        };

        Ok((
            TimingResult {
                time_real: Self::extract_time(command.get_command_line()),
                time_user: 0.0,
                time_system: 0.0,
            },
            status,
        ))
    }

    fn calibrate(&mut self) -> Result<()> {
        Ok(())
    }

    fn time_overhead(&self) -> Second {
        match &self.shell {
            None => 0.0,
            Some(shell) => Self::extract_time(shell),
        }
    }
}

#[test]
fn test_mock_executor_extract_time() {
    assert_eq!(MockExecutor::extract_time("sleep 0.1"), 0.1);
}

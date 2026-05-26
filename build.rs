use std::fs;
use std::path::{Path, PathBuf};

use clap_complete::{generate_to, Shell};

include!("src/cli.rs");

const FISH_COMMAND_COMPLETION: &str = "\
# Complete executables from $PATH for benchmark command arguments (see #872)
complete -c hyperfine -n \"__fish_use_subcommand\" -x -a \"(__fish_complete_command)\"
";

fn patch_fish_completions(path: &Path) {
    let mut content = fs::read_to_string(path).expect("read fish completions");
    if !content.contains("__fish_use_subcommand") {
        content.push_str(FISH_COMMAND_COMPLETION);
        fs::write(path, content).expect("write fish completions");
    }
}

fn main() {
    let var = std::env::var_os("SHELL_COMPLETIONS_DIR").or_else(|| std::env::var_os("OUT_DIR"));
    let outdir = match var {
        None => return,
        Some(outdir) => outdir,
    };
    fs::create_dir_all(&outdir).unwrap();

    let mut command = build_command();
    for shell in [
        Shell::Bash,
        Shell::Fish,
        Shell::Zsh,
        Shell::PowerShell,
        Shell::Elvish,
    ] {
        generate_to(shell, &mut command, "hyperfine", &outdir).unwrap();
        if shell == Shell::Fish {
            patch_fish_completions(&PathBuf::from(&outdir).join("hyperfine.fish"));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::FISH_COMMAND_COMPLETION;

    #[test]
    fn fish_command_completion_snippet_uses_path_executable_helper() {
        assert!(FISH_COMMAND_COMPLETION.contains("__fish_use_subcommand"));
        assert!(FISH_COMMAND_COMPLETION.contains("__fish_complete_command"));
    }
}

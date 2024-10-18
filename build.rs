use std::fs;
use std::path::PathBuf;
use std::error::Error;
use clap_complete::{generate_to, shells::Shell};

include!("src/cli.rs");

fn main() -> Result<(), Box<dyn Error>> {
    let outdir = get_output_dir()?;
    fs::create_dir_all(&outdir)?;

    let mut command = build_command();
    for shell in [
        Shell::Bash,
        Shell::Fish,
        Shell::Zsh,
        Shell::PowerShell,
        Shell::Elvish,
    ] {
        generate_to(shell, &mut command, "hyperfine", &outdir)
            .map_err(|e| format!("Failed to generate completions for {:?}: {}", shell, e))?;
    }

    println!("Completions generated successfully in {:?}", outdir);
    Ok(())
}

fn get_output_dir() -> Result<PathBuf, Box<dyn Error>> {
    std::env::var_os("SHELL_COMPLETIONS_DIR")
        .or_else(|| std::env::var_os("OUT_DIR"))
        .map(PathBuf::from)
        .ok_or_else(|| "Neither SHELL_COMPLETIONS_DIR nor OUT_DIR environment variable is set".into())
}

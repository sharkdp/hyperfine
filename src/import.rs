use crate::{benchmark::benchmark_result::BenchmarkResult, export::json::HyperfineSummary};
use clap::ArgMatches;
use std::fs;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Importer {}

impl Importer {
    pub fn from_cli_arguments(matches: &ArgMatches) -> Option<Vec<BenchmarkResult>> {
        match matches.get_one::<String>("import-json") {
            Some(file_name) => read_summary_from_file(file_name),
            None => None,
        }
    }
}

fn read_summary_from_file(file_name: &str) -> Option<Vec<BenchmarkResult>> {
    let file_content = match fs::read_to_string(file_name) {
        Ok(content) => content,
        Err(_) => {
            eprintln!("Unable to load previous run from file {}", file_name);
            return None;
        }
    };

    let hyperfine_summary = serde_json::from_str::<HyperfineSummary>(&file_content);
    match hyperfine_summary {
        Ok(summary) => Some(summary.results),
        Err(_) => None,
    }
}

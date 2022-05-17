use crate::benchmark::benchmark_result::BenchmarkResult;
use crate::benchmark::relative_speed::{self, BenchmarkResultWithRelativeSpeed};
use crate::output::format::format_duration_value;
use crate::util::units::Unit;

use super::Exporter;

use anyhow::{anyhow, Result};

#[derive(Default)]
pub struct AsciidocExporter {}

impl Exporter for AsciidocExporter {
    fn serialize(&self, results: &[BenchmarkResult], unit: Option<Unit>) -> Result<Vec<u8>> {
        let unit = if let Some(unit) = unit {
            // Use the given unit for all entries.
            unit
        } else if let Some(first_result) = results.first() {
            // Use the first BenchmarkResult entry to determine the unit for all entries.
            format_duration_value(first_result.mean, None).1
        } else {
            // Default to `Second`.
            Unit::Second
        };

        if let Some(annotated_results) = relative_speed::compute(results) {
            let mut res: Vec<u8> = Vec::new();
            res.append(&mut table_open());
            res.append(&mut table_startend());
            res.append(&mut table_header(unit));
            for result in annotated_results {
                res.push(b'\n');
                res.append(&mut table_row(&result, unit));
            }
            res.append(&mut table_startend());

            Ok(res)
        } else {
            Err(anyhow!(
                "Relative speed comparison is not available for Asciidoctor export."
            ))
        }
    }
}

fn table_open() -> Vec<u8> {
    "[cols=\"<,>,>,>,>\"]\n".bytes().collect()
}

fn table_startend() -> Vec<u8> {
    "|===\n".bytes().collect()
}

fn table_header(unittype: Unit) -> Vec<u8> {
    let unit_short_name = unittype.short_name();
    format!(
        "| Command \n| Mean [{unit}] \n| Min [{unit}] \n| Max [{unit}] \n| Relative \n",
        unit = unit_short_name
    )
    .into_bytes()
}

fn table_row(entry: &BenchmarkResultWithRelativeSpeed, unit: Unit) -> Vec<u8> {
    let result = &entry.result;
    let mean_str = format_duration_value(result.mean, Some(unit)).0;
    let stddev_str = if let Some(stddev) = result.stddev {
        format!(" ± {}", format_duration_value(stddev, Some(unit)).0)
    } else {
        "".into()
    };
    let min_str = format_duration_value(result.min, Some(unit)).0;
    let max_str = format_duration_value(result.max, Some(unit)).0;
    let rel_str = format!("{:.2}", entry.relative_speed);
    let rel_stddev_str = if entry.is_fastest {
        "".into()
    } else if let Some(stddev) = entry.relative_speed_stddev {
        format!(" ± {:.2}", stddev)
    } else {
        "".into()
    };

    format!(
        "| `{command}` \n\
         | {mean}{stddev} \n\
         | {min} \n\
         | {max} \n\
         | {rel}{rel_stddev} \n",
        command = result.command.replace('|', "\\|"),
        mean = mean_str,
        stddev = stddev_str,
        min = min_str,
        max = max_str,
        rel = rel_str,
        rel_stddev = rel_stddev_str
    )
    .into_bytes()
}

/// Ensure various options for the header generate correct results
#[test]
fn test_asciidoc_header() {
    let conms: Vec<u8> = "| Command \n| Mean [ms] \n| Min [ms] \n| Max [ms] \n| Relative \n"
        .bytes()
        .collect();
    let cons: Vec<u8> = "| Command \n| Mean [s] \n| Min [s] \n| Max [s] \n| Relative \n"
        .bytes()
        .collect();
    let genms = table_header(Unit::MilliSecond);
    let gens = table_header(Unit::Second);

    assert_eq!(conms, genms);
    assert_eq!(cons, gens);
}

/// Ensure each table row is generated properly
#[test]
fn test_asciidoc_table_row() {
    use std::collections::BTreeMap;

    let timing_result = BenchmarkResultWithRelativeSpeed {
        result: &BenchmarkResult {
            command: String::from("sleep 1"),
            mean: 0.10491992406666667,
            stddev: Some(0.00397851689425097),
            median: 0.10491992406666667,
            user: 0.005182013333333333,
            system: 0.0,
            min: 0.1003342584,
            max: 0.10745223440000001,
            times: Some(vec![0.1003342584, 0.10745223440000001, 0.10697327940000001]),
            exit_codes: vec![Some(0), Some(0), Some(0)],
            parameters: BTreeMap::new(),
        },
        relative_speed: 1.000,
        relative_speed_stddev: Option::from(1.03),
        is_fastest: true,
    };

    let formatted = String::from_utf8(table_row(&timing_result, Unit::MilliSecond)).unwrap();

    let formatted_expected = "| `sleep 1` \n\
         | 104.9 ± 4.0 \n\
         | 100.3 \n\
         | 107.5 \n\
         | 1.00 \n";

    assert_eq!(formatted_expected, formatted);

    let formatted_seconds = String::from_utf8(table_row(&timing_result, Unit::Second)).unwrap();
    let formatted_expected_seconds = "| `sleep 1` \n\
         | 0.105 ± 0.004 \n\
         | 0.100 \n\
         | 0.107 \n\
         | 1.00 \n";

    assert_eq!(formatted_expected_seconds, formatted_seconds);
}

/// Ensure commands get properly escaped
#[test]
fn test_asciidoc_table_row_command_escape() {
    use std::collections::BTreeMap;
    let benchmark_result = BenchmarkResultWithRelativeSpeed {
        result: &BenchmarkResult {
            command: String::from("sleep 1|"),
            mean: 0.10491992406666667,
            stddev: Some(0.00397851689425097),
            median: 0.10491992406666667,
            user: 0.005182013333333333,
            system: 0.0,
            min: 0.1003342584,
            max: 0.10745223440000001,
            times: Some(vec![0.1003342584, 0.10745223440000001, 0.10697327940000001]),
            exit_codes: vec![Some(0), Some(0), Some(0)],
            parameters: BTreeMap::new(),
        },
        relative_speed: 1.000,
        relative_speed_stddev: Option::from(1.03),
        is_fastest: true,
    };
    let expected = String::from_utf8(
        format!(
            "| `sleep 1\\|` \n\
         | {} ± {} \n\
         | {} \n\
         | {} \n\
         | {:.2} \n",
            Unit::Second.format(benchmark_result.result.mean),
            Unit::Second.format(benchmark_result.result.stddev.unwrap()),
            Unit::Second.format(benchmark_result.result.min),
            Unit::Second.format(benchmark_result.result.max),
            benchmark_result.relative_speed
        )
        .into_bytes(),
    );
    let generated_seconds = String::from_utf8(table_row(&benchmark_result, Unit::Second));

    assert_eq!(expected, generated_seconds);
}

/// Integration test
#[test]
fn test_asciidoc() {
    use std::collections::BTreeMap;
    let exporter = AsciidocExporter::default();
    // NOTE: results are fabricated, unlike above
    let results = vec![
        BenchmarkResult {
            command: String::from("FOO=1 BAR=2 command | 1"),
            mean: 1.0,
            stddev: Some(2.0),
            median: 1.0,
            user: 3.0,
            system: 4.0,
            min: 5.0,
            max: 6.0,
            times: Some(vec![7.0, 8.0, 9.0]),
            exit_codes: vec![Some(0), Some(0), Some(0)],
            parameters: {
                let mut params = BTreeMap::new();
                params.insert("foo".into(), "1".into());
                params.insert("bar".into(), "2".into());
                params
            },
        },
        BenchmarkResult {
            command: String::from("FOO=1 BAR=7 command | 2"),
            mean: 11.0,
            stddev: Some(12.0),
            median: 11.0,
            user: 13.0,
            system: 14.0,
            min: 15.0,
            max: 16.0,
            times: Some(vec![17.0, 18.0, 19.0]),
            exit_codes: vec![Some(0), Some(0), Some(0)],
            parameters: {
                let mut params = BTreeMap::new();
                params.insert("foo".into(), "1".into());
                params.insert("bar".into(), "7".into());
                params
            },
        },
    ];
    // NOTE: only testing with s, s/ms is tested elsewhere
    let expected: String = String::from(
        "[cols=\"<,>,>,>,>\"]\n\
         |===\n\
         | Command \n\
         | Mean [s] \n\
         | Min [s] \n\
         | Max [s] \n\
         | Relative \n\
         \n\
         | `FOO=1 BAR=2 command \\| 1` \n\
         | 1.000 ± 2.000 \n\
         | 5.000 \n\
         | 6.000 \n\
         | 1.00 \n\
         \n\
         | `FOO=1 BAR=7 command \\| 2` \n\
         | 11.000 ± 12.000 \n\
         | 15.000 \n\
         | 16.000 \n\
         | 11.00 ± 25.06 \n\
         |===\n\
         ",
    );
    let given =
        String::from_utf8(exporter.serialize(&results, Some(Unit::Second)).unwrap()).unwrap();

    assert_eq!(expected, given);
}

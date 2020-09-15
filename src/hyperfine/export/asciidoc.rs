use super::Exporter;

use crate::hyperfine::format::format_duration_value;
use crate::hyperfine::types::BenchmarkResult;
use crate::hyperfine::units::Unit;

use std::io::Result;

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

        let mut res: Vec<u8> = Vec::new();
        res.append(&mut table_open());
        res.append(&mut table_startend());
        res.append(&mut table_header(unit));
        for result in results {
            res.push(b'\n');
            res.append(&mut table_row(result, unit));
        }
        res.append(&mut table_startend());

        Ok(res)
    }
}

fn table_open() -> Vec<u8> {
    "[cols=\"<,>,>\"]\n".bytes().collect()
}

fn table_startend() -> Vec<u8> {
    "|===\n".bytes().collect()
}

fn table_header(unittype: Unit) -> Vec<u8> {
    let unit_short_name = unittype.short_name();
    format!(
        "| Command | Mean [{unit}] | Min…Max [{unit}]\n",
        unit = unit_short_name
    )
    .into_bytes()
}

fn table_row(entry: &BenchmarkResult, unit: Unit) -> Vec<u8> {
    let form = |val| format_duration_value(val, Some(unit));
    format!(
        "| `{}`\n\
         | {} ± {}\n\
         | {}…{}\n",
        entry.command.replace("|", "\\|"),
        form(entry.mean).0,
        form(entry.stddev).0,
        form(entry.min).0,
        form(entry.max).0
    )
    .into_bytes()
}

/// Ensure various options for the header generate correct results
#[test]
fn test_asciidoc_header() {
    let conms: Vec<u8> = "| Command | Mean [ms] | Min…Max [ms]\n".bytes().collect();
    let cons: Vec<u8> = "| Command | Mean [s] | Min…Max [s]\n".bytes().collect();
    let genms = table_header(Unit::MilliSecond);
    let gens = table_header(Unit::Second);

    assert_eq!(conms, genms);
    assert_eq!(cons, gens);
}

/// Ensure each table row is generated properly
#[test]
fn test_asciidoc_table_row() {
    use std::collections::BTreeMap;
    let result = BenchmarkResult::new(
        String::from("sleep 1"), // command
        0.10491992406666667,     // mean
        0.00397851689425097,     // stddev
        0.10491992406666667,     // median
        0.005182013333333333,    // user
        0.0,                     // system
        0.1003342584,            // min
        0.10745223440000001,     // max
        vec![
            // times
            0.1003342584,
            0.10745223440000001,
            0.10697327940000001,
        ],
        BTreeMap::new(), // param
    );

    let expms = format!(
        "| `{}`\n\
         | {} ± {}\n\
         | {}…{}\n",
        result.command,
        Unit::MilliSecond.format(result.mean),
        Unit::MilliSecond.format(result.stddev),
        Unit::MilliSecond.format(result.min),
        Unit::MilliSecond.format(result.max)
    )
    .into_bytes();
    let exps = format!(
        "| `{}`\n\
         | {} ± {}\n\
         | {}…{}\n",
        result.command,
        Unit::Second.format(result.mean),
        Unit::Second.format(result.stddev),
        Unit::Second.format(result.min),
        Unit::Second.format(result.max)
    )
    .into_bytes();

    let genms = table_row(&result, Unit::MilliSecond);
    let gens = table_row(&result, Unit::Second);

    assert_eq!(expms, genms);
    assert_eq!(exps, gens);
}

/// Ensure commands get properly escaped
#[test]
fn test_asciidoc_table_row_command_escape() {
    use std::collections::BTreeMap;
    let result = BenchmarkResult::new(
        String::from("sleep 1|"), // command
        0.10491992406666667,      // mean
        0.00397851689425097,      // stddev
        0.10491992406666667,      // median
        0.005182013333333333,     // user
        0.0,                      // system
        0.1003342584,             // min
        0.10745223440000001,      // max
        vec![
            // times
            0.1003342584,
            0.10745223440000001,
            0.10697327940000001,
        ],
        BTreeMap::new(), // param
    );
    let exps = format!(
        "| `sleep 1\\|`\n\
         | {} ± {}\n\
         | {}…{}\n",
        Unit::Second.format(result.mean),
        Unit::Second.format(result.stddev),
        Unit::Second.format(result.min),
        Unit::Second.format(result.max)
    )
    .into_bytes();
    let gens = table_row(&result, Unit::Second);

    assert_eq!(exps, gens);
}

/// Integration test
#[test]
fn test_asciidoc() {
    use std::collections::BTreeMap;
    let exporter = AsciidocExporter::default();
    // NOTE: results are fabricated, unlike above
    let results = vec![
        BenchmarkResult::new(
            String::from("FOO=1 BAR=2 command | 1"),
            1.0,
            2.0,
            1.0,
            3.0,
            4.0,
            5.0,
            6.0,
            vec![7.0, 8.0, 9.0],
            {
                let mut params = BTreeMap::new();
                params.insert("foo".into(), "1".into());
                params.insert("bar".into(), "2".into());
                params
            },
        ),
        BenchmarkResult::new(
            String::from("FOO=1 BAR=7 command | 2"),
            11.0,
            12.0,
            11.0,
            13.0,
            14.0,
            15.0,
            16.0,
            vec![17.0, 18.0, 19.0],
            {
                let mut params = BTreeMap::new();
                params.insert("foo".into(), "1".into());
                params.insert("bar".into(), "7".into());
                params
            },
        ),
    ];
    // NOTE: only testing with s, s/ms is tested elsewhere
    let exps: String = String::from(
        "[cols=\"<,>,>\"]\n\
         |===\n\
         | Command | Mean [s] | Min…Max [s]\n\
         \n\
         | `FOO=1 BAR=2 command \\| 1`\n\
         | 1.000 ± 2.000\n\
         | 5.000…6.000\n\
         \n\
         | `FOO=1 BAR=7 command \\| 2`\n\
         | 11.000 ± 12.000\n\
         | 15.000…16.000\n\
         |===\n\
         ",
    );
    let gens =
        String::from_utf8(exporter.serialize(&results, Some(Unit::Second)).unwrap()).unwrap();

    assert_eq!(exps, gens);
}

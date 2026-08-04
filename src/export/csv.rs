use std::borrow::Cow;

use csv::WriterBuilder;

use super::Exporter;
use crate::benchmark::benchmark_result::BenchmarkResult;
use crate::options::SortOrder;
use crate::util::units::Unit;

use anyhow::Result;

/// Prefixes a value with a single quote if it begins with a character that
/// spreadsheet applications (Excel, LibreOffice Calc, Google Sheets)
/// interpret as the start of a formula, so the value is displayed as
/// literal text instead of being evaluated. See CWE-1236.
fn neutralize_formula(value: &[u8]) -> Cow<'_, [u8]> {
    match value.first() {
        Some(b'=' | b'+' | b'-' | b'@' | b'\t' | b'\r') => {
            let mut escaped = Vec::with_capacity(value.len() + 1);
            escaped.push(b'\'');
            escaped.extend_from_slice(value);
            Cow::Owned(escaped)
        }
        _ => Cow::Borrowed(value),
    }
}

#[derive(Default)]
pub struct CsvExporter {}

impl Exporter for CsvExporter {
    fn serialize(
        &self,
        results: &[BenchmarkResult],
        _unit: Option<Unit>,
        _sort_order: SortOrder,
    ) -> Result<Vec<u8>> {
        let mut writer = WriterBuilder::new().from_writer(vec![]);

        {
            let mut headers: Vec<Cow<[u8]>> = [
                // The list of times and exit codes cannot be exported to the CSV file - omit them.
                "command", "mean", "stddev", "median", "user", "system", "min", "max",
            ]
            .iter()
            .map(|x| Cow::Borrowed(x.as_bytes()))
            .collect();
            if let Some(res) = results.first() {
                for param_name in res.parameters.keys() {
                    headers.push(Cow::Owned(format!("parameter_{param_name}").into_bytes()));
                }
            }
            writer.write_record(headers)?;
        }

        for res in results {
            let mut fields = vec![neutralize_formula(res.command.as_bytes())];
            for f in &[
                res.mean,
                res.stddev.unwrap_or(0.0),
                res.median,
                res.user,
                res.system,
                res.min,
                res.max,
            ] {
                fields.push(Cow::Owned(f.to_string().into_bytes()))
            }
            for v in res.parameters.values() {
                fields.push(neutralize_formula(v.as_bytes()))
            }
            writer.write_record(fields)?;
        }

        Ok(writer.into_inner()?)
    }
}

#[test]
fn test_csv() {
    use std::collections::BTreeMap;
    let exporter = CsvExporter::default();

    let results = vec![
        BenchmarkResult {
            command: String::from("command_a"),
            command_with_unused_parameters: String::from("command_a"),
            mean: 1.0,
            stddev: Some(2.0),
            median: 1.0,
            user: 3.0,
            system: 4.0,
            min: 5.0,
            max: 6.0,
            times: Some(vec![7.0, 8.0, 9.0]),
            memory_usage_byte: None,
            exit_codes: vec![Some(0), Some(0), Some(0)],
            parameters: {
                let mut params = BTreeMap::new();
                params.insert("foo".into(), "one".into());
                params.insert("bar".into(), "two".into());
                params
            },
        },
        BenchmarkResult {
            command: String::from("command_b"),
            command_with_unused_parameters: String::from("command_b"),
            mean: 11.0,
            stddev: Some(12.0),
            median: 11.0,
            user: 13.0,
            system: 14.0,
            min: 15.0,
            max: 16.5,
            times: Some(vec![17.0, 18.0, 19.0]),
            memory_usage_byte: None,
            exit_codes: vec![Some(0), Some(0), Some(0)],
            parameters: {
                let mut params = BTreeMap::new();
                params.insert("foo".into(), "one".into());
                params.insert("bar".into(), "seven".into());
                params
            },
        },
    ];

    let actual = String::from_utf8(
        exporter
            .serialize(&results, Some(Unit::Second), SortOrder::Command)
            .unwrap(),
    )
    .unwrap();

    insta::assert_snapshot!(actual, @r#"
    command,mean,stddev,median,user,system,min,max,parameter_bar,parameter_foo
    command_a,1,2,1,3,4,5,6,two,one
    command_b,11,12,11,13,14,15,16.5,seven,one
    "#);
}

/// Regression test for CWE-1236: a command or parameter value beginning
/// with a spreadsheet formula character must not be written to the CSV
/// file as-is, since Excel, LibreOffice Calc, and Google Sheets would
/// evaluate it as a formula rather than displaying it as text.
#[test]
fn test_csv_formula_injection_is_neutralized() {
    use std::collections::BTreeMap;
    let exporter = CsvExporter::default();

    let results = vec![BenchmarkResult {
        command: String::from("=1+1"),
        command_with_unused_parameters: String::from("=1+1"),
        mean: 1.0,
        stddev: Some(2.0),
        median: 1.0,
        user: 3.0,
        system: 4.0,
        min: 5.0,
        max: 6.0,
        times: Some(vec![7.0, 8.0, 9.0]),
        memory_usage_byte: None,
        exit_codes: vec![Some(0), Some(0), Some(0)],
        parameters: {
            let mut params = BTreeMap::new();
            params.insert("payload".into(), "@SUM(1)".into());
            params
        },
    }];

    let actual = String::from_utf8(
        exporter
            .serialize(&results, Some(Unit::Second), SortOrder::Command)
            .unwrap(),
    )
    .unwrap();

    insta::assert_snapshot!(actual, @r#"
    command,mean,stddev,median,user,system,min,max,parameter_payload
    '=1+1,1,2,1,3,4,5,6,'@SUM(1)
    "#);
}

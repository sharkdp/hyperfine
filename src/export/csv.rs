use std::borrow::Cow;

use csv::WriterBuilder;

use super::Exporter;
use crate::benchmark::benchmark_result::BenchmarkResult;
use crate::options::SortOrder;
use crate::util::units::Unit;

use anyhow::Result;

#[derive(Default)]
pub struct CsvExporter {}

/// Sanitizes a CSV field value to prevent formula injection (CWE-1236).
///
/// Spreadsheet applications (Excel, LibreOffice Calc, Google Sheets) interpret
/// cells beginning with `=`, `+`, `-`, or `@` as formulas. When the value
/// originates from user-controlled input (e.g. a benchmarked command name or
/// parameter value), this can lead to formula injection. Prefixing such values
/// with a single quote (`'`) instructs spreadsheet applications to treat the
/// content as literal text.
fn sanitize_csv_value(value: &str) -> Cow<'_, [u8]> {
    match value.chars().next() {
        Some('=') | Some('+') | Some('-') | Some('@') | Some('\t') | Some('\r') => {
            let mut sanitized = String::from("'");
            sanitized.push_str(value);
            Cow::Owned(sanitized.into_bytes())
        }
        _ => Cow::Borrowed(value.as_bytes()),
    }
}

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
            let mut fields = vec![sanitize_csv_value(&res.command)];
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
                fields.push(sanitize_csv_value(v))
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

#[test]
fn test_csv_formula_injection_sanitization() {
    use std::collections::BTreeMap;
    let exporter = CsvExporter::default();

    let results = vec![BenchmarkResult {
        command: String::from("sleep 0.1"),
        command_with_unused_parameters: String::from("sleep 0.1"),
        mean: 0.1,
        stddev: None,
        median: 0.1,
        user: 0.0,
        system: 0.0,
        min: 0.1,
        max: 0.1,
        times: None,
        memory_usage_byte: None,
        exit_codes: vec![Some(0)],
        parameters: {
            let mut params = BTreeMap::new();
            params.insert("payload".into(), "=1+1".into());
            params.insert("safe_param".into(), "value".into());
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
    command,mean,stddev,median,user,system,min,max,parameter_payload,parameter_safe_param
    sleep 0.1,0.1,0,0.1,0,0,0.1,0.1,'=1+1,value
    "#);
}

#[test]
fn test_sanitize_csv_value() {
    // Values that begin with formula characters should be prefixed with a single quote
    assert_eq!(
        sanitize_csv_value("=cmd|' /C calc'!A0").as_ref(),
        b"'=cmd|' /C calc'!A0".as_slice()
    );
    assert_eq!(sanitize_csv_value("+1+1").as_ref(), b"'+1+1".as_slice());
    assert_eq!(sanitize_csv_value("-1+1").as_ref(), b"'-1+1".as_slice());
    assert_eq!(
        sanitize_csv_value("@SUM(A1:A5)").as_ref(),
        b"'@SUM(A1:A5)".as_slice()
    );
    assert_eq!(
        sanitize_csv_value("\tformula").as_ref(),
        b"'\tformula".as_slice()
    );
    assert_eq!(
        sanitize_csv_value("\rformula").as_ref(),
        b"'\rformula".as_slice()
    );

    // Safe values should pass through unchanged
    assert_eq!(
        sanitize_csv_value("echo hello").as_ref(),
        "echo hello".as_bytes()
    );
    assert_eq!(
        sanitize_csv_value("value").as_ref(),
        b"value".as_slice()
    );
    assert_eq!(sanitize_csv_value("").as_ref(), b"".as_slice());
    assert_eq!(
        sanitize_csv_value("normal-param").as_ref(),
        b"normal-param".as_slice()
    );
}

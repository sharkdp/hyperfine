use std::borrow::Cow;

use csv::WriterBuilder;

use super::Exporter;
use crate::benchmark::benchmark_result::BenchmarkResult;
use crate::options::SortOrder;
use crate::util::units::Unit;

use anyhow::Result;

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
            let mut fields = vec![Cow::Borrowed(res.command.as_bytes())];
            for f in &[
                res.mean(),
                res.stddev().unwrap_or(0.0),
                res.median(),
                res.user,
                res.system,
                res.min(),
                res.max(),
            ] {
                fields.push(Cow::Owned(f.to_string().into_bytes()))
            }
            for v in res.parameters.values() {
                fields.push(Cow::Borrowed(v.as_bytes()))
            }
            writer.write_record(fields)?;
        }

        Ok(writer.into_inner()?)
    }
}

#[cfg(test)]
use crate::benchmark::benchmark_result::BenchmarkRun;

#[test]
fn test_csv() {
    use std::collections::BTreeMap;
    let exporter = CsvExporter::default();

    let results = vec![
        BenchmarkResult {
            command: String::from("command_a"),
            command_with_unused_parameters: String::from("command_a"),
            user: 3.0,
            system: 4.0,
            runs: vec![
                BenchmarkRun {
                    wall_clock_time: 7.0,
                },
                BenchmarkRun {
                    wall_clock_time: 8.0,
                },
                BenchmarkRun {
                    wall_clock_time: 12.0,
                },
            ],
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
            user: 13.0,
            system: 14.0,
            runs: vec![
                BenchmarkRun {
                    wall_clock_time: 17.0,
                },
                BenchmarkRun {
                    wall_clock_time: 18.0,
                },
                BenchmarkRun {
                    wall_clock_time: 19.0,
                },
            ],
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
    command_a,9,2.6457513110645907,8,3,4,7,12,two,one
    command_b,18,1,18,13,14,17,19,seven,one
    "#);
}

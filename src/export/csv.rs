use std::borrow::Cow;

use csv::WriterBuilder;

use super::Exporter;
use crate::benchmark::benchmark_result::BenchmarkResult;
use crate::options::SortOrder;
use crate::quantity::{TimeQuantity, TimeUnit};

use anyhow::Result;

#[derive(Default)]
pub struct CsvExporter {}

impl Exporter for CsvExporter {
    fn serialize(
        &self,
        results: &[BenchmarkResult],
        _time_unit: Option<TimeUnit>,
        _sort_order: SortOrder,
    ) -> Result<Vec<u8>> {
        const CSV_UNIT: TimeUnit = TimeUnit::Second;
        const CSV_PRECISION: usize = 6;

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
                res.mean_wall_clock_time(),
                res.measurements.stddev().unwrap_or_default(),
                res.measurements.median(),
                res.measurements.time_user_mean(),
                res.measurements.time_system_mean(),
                res.measurements.min(),
                res.measurements.max(),
            ] {
                fields.push(Cow::Owned(
                    f.format(CSV_UNIT, Some(CSV_PRECISION)).into_bytes(),
                ))
            }
            for v in res.parameters.values() {
                fields.push(Cow::Borrowed(v.value.as_bytes()))
            }
            writer.write_record(fields)?;
        }

        Ok(writer.into_inner()?)
    }
}

#[test]
fn test_csv() {
    use crate::benchmark::benchmark_result::Parameter;
    use crate::benchmark::measurement::{Measurement, Measurements};
    use crate::quantity::{byte, second, Information, Time, TimeQuantity};

    use std::collections::BTreeMap;
    use std::process::ExitStatus;

    let exporter = CsvExporter::default();

    let results = vec![
        BenchmarkResult {
            command: String::from("command_a"),
            measurements: Measurements::new(vec![
                Measurement {
                    time_wall_clock: Time::new::<second>(7.0),
                    time_user: Time::new::<second>(7.0),
                    time_system: Time::zero(),
                    peak_memory_usage: Information::new::<byte>(1024),
                    exit_status: ExitStatus::default(),
                },
                Measurement {
                    time_wall_clock: Time::new::<second>(8.0),
                    time_user: Time::new::<second>(8.0),
                    time_system: Time::zero(),
                    peak_memory_usage: Information::new::<byte>(1024),
                    exit_status: ExitStatus::default(),
                },
                Measurement {
                    time_wall_clock: Time::new::<second>(12.0),
                    time_user: Time::new::<second>(12.0),
                    time_system: Time::zero(),
                    peak_memory_usage: Information::new::<byte>(1024),
                    exit_status: ExitStatus::default(),
                },
            ]),
            parameters: {
                let mut params = BTreeMap::new();
                params.insert(
                    "foo".into(),
                    Parameter {
                        value: "one".into(),
                        is_unused: false,
                    },
                );
                params.insert(
                    "bar".into(),
                    Parameter {
                        value: "two".into(),
                        is_unused: false,
                    },
                );
                params
            },
        },
        BenchmarkResult {
            command: String::from("command_b"),
            measurements: Measurements::new(vec![
                Measurement {
                    time_wall_clock: Time::new::<second>(17.0),
                    time_user: Time::new::<second>(17.0),
                    time_system: Time::zero(),
                    peak_memory_usage: Information::new::<byte>(1024),
                    exit_status: ExitStatus::default(),
                },
                Measurement {
                    time_wall_clock: Time::new::<second>(18.0),
                    time_user: Time::new::<second>(18.0),
                    time_system: Time::zero(),
                    peak_memory_usage: Information::new::<byte>(1024),
                    exit_status: ExitStatus::default(),
                },
                Measurement {
                    time_wall_clock: Time::new::<second>(19.0),
                    time_user: Time::new::<second>(19.0),
                    time_system: Time::zero(),
                    peak_memory_usage: Information::new::<byte>(1024),
                    exit_status: ExitStatus::default(),
                },
            ]),
            parameters: {
                let mut params = BTreeMap::new();
                params.insert(
                    "foo".into(),
                    Parameter {
                        value: "one".into(),
                        is_unused: false,
                    },
                );
                params.insert(
                    "bar".into(),
                    Parameter {
                        value: "seven".into(),
                        is_unused: false,
                    },
                );
                params
            },
        },
    ];

    let actual = String::from_utf8(
        exporter
            .serialize(&results, Some(TimeUnit::Second), SortOrder::Command)
            .unwrap(),
    )
    .unwrap();

    insta::assert_snapshot!(actual, @r#"
    command,mean,stddev,median,user,system,min,max,parameter_bar,parameter_foo
    command_a,9.000000,2.645751,8.000000,9.000000,0.000000,7.000000,12.000000,two,one
    command_b,18.000000,1.000000,18.000000,18.000000,0.000000,17.000000,19.000000,seven,one
    "#);
}

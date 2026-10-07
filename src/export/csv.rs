use csv::WriterBuilder;

use super::Exporter;
use crate::benchmark::benchmark_result::BenchmarkResult;
use crate::metric::MetricSelection;

use anyhow::{Context, Result};

#[derive(Default)]
pub struct CsvExporter {}

impl Exporter for CsvExporter {
    fn serialize(&self, results: &[BenchmarkResult], primary: MetricSelection) -> Result<Vec<u8>> {
        // CSV never chooses units automatically: an omitted suffix means base units.
        let unit = primary.csv_unit();
        let mut writer = WriterBuilder::new().from_writer(vec![]);
        let mut headers: Vec<String> = [
            "command", "metric", "unit", "mean", "stddev", "median", "min", "max",
        ]
        .iter()
        .copied()
        .map(str::to_owned)
        .collect();
        if let Some(result) = results.first() {
            headers.extend(
                result
                    .parameters
                    .keys()
                    .map(|name| format!("parameter_{name}")),
            );
        }
        writer.write_record(headers)?;

        for result in results {
            let stats = primary
                .metric
                .summarize(&result.measurements)
                .with_context(|| {
                    format!(
                        "Metric '{}' is unavailable for '{}'",
                        primary.metric.name(),
                        result.get_name()
                    )
                })?;
            let mut fields = vec![
                result.get_name().to_owned(),
                primary.metric.name().to_owned(),
                unit.symbol.to_owned(),
            ];
            // Preserve numeric precision rather than using rounded display values.
            fields.extend(
                [
                    Some(stats.mean),
                    stats.stddev,
                    Some(stats.median),
                    Some(stats.min),
                    Some(stats.max),
                ]
                .map(|value| {
                    value
                        .map(|value| (value / unit.scale).to_string())
                        .unwrap_or_default()
                }),
            );
            fields.extend(
                result
                    .parameters
                    .values()
                    .map(|parameter| parameter.value.clone()),
            );
            writer.write_record(fields)?;
        }

        Ok(writer.into_inner()?)
    }
}

#[test]
fn test_csv() {
    use crate::benchmark::benchmark_result::Parameter;
    use crate::benchmark::measurement::{Measurement, Measurements};
    use crate::quantity::{byte, second, Information, Time, Zero};

    use std::collections::BTreeMap;
    use std::process::ExitStatus;

    let exporter = CsvExporter::default();

    let results = vec![
        BenchmarkResult {
            environment: BTreeMap::new(),
            command: String::from("echo command_a"),
            name: Some(String::from("command_a")),
            display_name: String::from("command_a"),
            measurements: Measurements::new(vec![
                Measurement {
                    time_wall_clock: Time::new::<second>(7.0),
                    time_cpu: Time::new::<second>(7.0),
                    time_user: Time::new::<second>(7.0),
                    time_system: Time::zero(),
                    memory_peak_resident: Some(Information::new::<byte>(1024.)),
                    hardware_counters: Default::default(),
                    exit_status: ExitStatus::default(),
                },
                Measurement {
                    time_wall_clock: Time::new::<second>(8.0),
                    time_cpu: Time::new::<second>(8.0),
                    time_user: Time::new::<second>(8.0),
                    time_system: Time::zero(),
                    memory_peak_resident: Some(Information::new::<byte>(1024.)),
                    hardware_counters: Default::default(),
                    exit_status: ExitStatus::default(),
                },
                Measurement {
                    time_wall_clock: Time::new::<second>(12.0),
                    time_cpu: Time::new::<second>(12.0),
                    time_user: Time::new::<second>(12.0),
                    time_system: Time::zero(),
                    memory_peak_resident: Some(Information::new::<byte>(1024.)),
                    hardware_counters: Default::default(),
                    exit_status: ExitStatus::default(),
                },
            ]),
            parameters: {
                let mut params = BTreeMap::new();
                params.insert(
                    "foo".into(),
                    Parameter {
                        value: "one".into(),
                    },
                );
                params.insert(
                    "bar".into(),
                    Parameter {
                        value: "two".into(),
                    },
                );
                params
            },
        },
        BenchmarkResult {
            environment: BTreeMap::new(),
            command: String::from("command_b"),
            name: None,
            display_name: String::from("command_b"),
            measurements: Measurements::new(vec![
                Measurement {
                    time_wall_clock: Time::new::<second>(17.0),
                    time_cpu: Time::new::<second>(17.0),
                    time_user: Time::new::<second>(17.0),
                    time_system: Time::zero(),
                    memory_peak_resident: Some(Information::new::<byte>(1024.)),
                    hardware_counters: Default::default(),
                    exit_status: ExitStatus::default(),
                },
                Measurement {
                    time_wall_clock: Time::new::<second>(18.0),
                    time_cpu: Time::new::<second>(18.0),
                    time_user: Time::new::<second>(18.0),
                    time_system: Time::zero(),
                    memory_peak_resident: Some(Information::new::<byte>(1024.)),
                    hardware_counters: Default::default(),
                    exit_status: ExitStatus::default(),
                },
                Measurement {
                    time_wall_clock: Time::new::<second>(19.0),
                    time_cpu: Time::new::<second>(19.0),
                    time_user: Time::new::<second>(19.0),
                    time_system: Time::zero(),
                    memory_peak_resident: Some(Information::new::<byte>(1024.)),
                    hardware_counters: Default::default(),
                    exit_status: ExitStatus::default(),
                },
            ]),
            parameters: {
                let mut params = BTreeMap::new();
                params.insert(
                    "foo".into(),
                    Parameter {
                        value: "one".into(),
                    },
                );
                params.insert(
                    "bar".into(),
                    Parameter {
                        value: "seven".into(),
                    },
                );
                params
            },
        },
    ];

    let actual = String::from_utf8(
        exporter
            .serialize(&results, MetricSelection::default())
            .unwrap(),
    )
    .unwrap();

    insta::assert_snapshot!(actual, @r#"
    command,metric,unit,mean,stddev,median,min,max,parameter_bar,parameter_foo
    command_a,time_wall_clock,s,9,2.6457513110645907,8,7,12,two,one
    command_b,time_wall_clock,s,18,1,18,17,19,seven,one
    "#);

    for (selection, unit, mean) in [
        ("memory_peak_resident", "B", "1024"),
        ("memory_peak_resident:KiB", "KiB", "1"),
    ] {
        let output = exporter
            .serialize(&results, MetricSelection::parse_list(selection).unwrap()[0])
            .unwrap();
        let mut reader = csv::Reader::from_reader(output.as_slice());
        let row = reader.records().next().unwrap().unwrap();
        assert_eq!(&row[1], "memory_peak_resident");
        assert_eq!(&row[2], unit);
        assert_eq!(&row[3], mean);
    }
}

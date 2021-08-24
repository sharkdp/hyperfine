use std::borrow::Cow;
use std::io::{Error, ErrorKind, Result};

use csv::WriterBuilder;

use super::Exporter;
use crate::benchmark_result::BenchmarkResult;
use crate::units::Unit;

#[derive(Default)]
pub struct CsvExporter {}

impl Exporter for CsvExporter {
    fn serialize(&self, results: &[BenchmarkResult], _unit: Option<Unit>) -> Result<Vec<u8>> {
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
                    headers.push(Cow::Owned(format!("parameter_{}", param_name).into_bytes()));
                }
            }
            writer.write_record(headers)?;
        }

        for res in results {
            let mut fields = vec![Cow::Borrowed(res.command.as_bytes())];
            for f in &[
                res.mean, res.stddev, res.median, res.user, res.system, res.min, res.max,
            ] {
                fields.push(Cow::Owned(f.to_string().into_bytes()))
            }
            for v in res.parameters.values() {
                fields.push(Cow::Borrowed(v.as_bytes()))
            }
            writer.write_record(fields)?;
        }

        writer
            .into_inner()
            .map_err(|e| Error::new(ErrorKind::Other, e))
    }
}

#[test]
fn test_csv() {
    use std::collections::BTreeMap;
    let exporter = CsvExporter::default();

    // NOTE: results are fabricated
    let results = vec![
        BenchmarkResult::new(
            String::from("FOO=one BAR=two command | 1"),
            1.0,
            2.0,
            1.0,
            3.0,
            4.0,
            5.0,
            6.0,
            vec![7.0, 8.0, 9.0],
            vec![Some(0), Some(0), Some(0)],
            {
                let mut params = BTreeMap::new();
                params.insert("foo".into(), "one".into());
                params.insert("bar".into(), "two".into());
                params
            },
        ),
        BenchmarkResult::new(
            String::from("FOO=one BAR=seven command | 2"),
            11.0,
            12.0,
            11.0,
            13.0,
            14.0,
            15.0,
            16.5,
            vec![17.0, 18.0, 19.0],
            vec![Some(0), Some(0), Some(0)],
            {
                let mut params = BTreeMap::new();
                params.insert("foo".into(), "one".into());
                params.insert("bar".into(), "seven".into());
                params
            },
        ),
    ];
    let exps: String = String::from(
        "command,mean,stddev,median,user,system,min,max,parameter_bar,parameter_foo\n\
        FOO=one BAR=two command | 1,1,2,1,3,4,5,6,two,one\n\
        FOO=one BAR=seven command | 2,11,12,11,13,14,15,16.5,seven,one\n\
        ",
    );
    let gens =
        String::from_utf8(exporter.serialize(&results, Some(Unit::Second)).unwrap()).unwrap();

    assert_eq!(exps, gens);
}

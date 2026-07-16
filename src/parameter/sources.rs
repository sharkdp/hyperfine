use std::{collections::BTreeMap, io, path::Path};

use clap::ArgMatches;

use super::{
    file_values::FileValues,
    product::{ParameterProduct, ParameterSource, ValuesSource},
    tokenize::tokenize,
};

/// Build a lazy parameter product while preserving the order in which
/// `--parameter-list` and `--parameter-file` occurrences appeared on the
/// command line.
pub fn product_from_cli_arguments<'a>(matches: &'a ArgMatches) -> io::Result<ParameterProduct<'a>> {
    let mut indexed_sources = Vec::new();

    for (index, name, values) in argument_pairs(matches, "parameter-list") {
        indexed_sources.push((
            index,
            ParameterSource::new(name, ValuesSource::inline(tokenize(values))),
        ));
    }

    for (index, name, path) in argument_pairs(matches, "parameter-file") {
        indexed_sources.push((
            index,
            ParameterSource::new(name, ValuesSource::file(FileValues::open(Path::new(path))?)),
        ));
    }

    indexed_sources.sort_by_key(|(index, _)| *index);

    let mut name_counts = BTreeMap::new();
    for (_, source) in &indexed_sources {
        *name_counts.entry(source.name()).or_insert(0_usize) += 1;
    }
    let duplicates = name_counts
        .into_iter()
        .filter_map(|(name, count)| (count > 1).then_some(name))
        .collect::<Vec<_>>();
    if !duplicates.is_empty() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("Duplicate parameter names: {}", duplicates.join(", ")),
        ));
    }

    ParameterProduct::new(
        indexed_sources
            .into_iter()
            .map(|(_, source)| source)
            .collect(),
    )
}

fn argument_pairs<'a>(matches: &'a ArgMatches, id: &str) -> Vec<(usize, &'a str, &'a str)> {
    let values = matches
        .get_many::<String>(id)
        .into_iter()
        .flatten()
        .map(String::as_str)
        .collect::<Vec<_>>();
    let indices = matches
        .indices_of(id)
        .into_iter()
        .flatten()
        .collect::<Vec<_>>();

    debug_assert_eq!(values.len(), indices.len());
    debug_assert_eq!(values.len() % 2, 0);

    indices
        .chunks_exact(2)
        .zip(values.chunks_exact(2))
        .map(|(indices, values)| (indices[0], values[0], values[1]))
        .collect()
}

#[cfg(test)]
mod tests {
    use std::io::Write;

    use tempfile::NamedTempFile;

    use super::product_from_cli_arguments;
    use crate::{cli::get_cli_arguments, parameter::ParameterValue};

    fn text_values(parameters: Vec<(&str, ParameterValue)>) -> Vec<(&str, String)> {
        parameters
            .into_iter()
            .map(|(name, value)| match value {
                ParameterValue::Text(value) => (name, value),
                ParameterValue::Numeric(_) => unreachable!(),
            })
            .collect()
    }

    #[test]
    fn preserves_interspersed_list_and_file_order() {
        let mut first_file = NamedTempFile::new().unwrap();
        first_file.write_all(b"x\ny\n").unwrap();
        first_file.flush().unwrap();

        let mut last_file = NamedTempFile::new().unwrap();
        last_file.write_all(b"q\nr\n").unwrap();
        last_file.flush().unwrap();

        let matches = get_cli_arguments([
            "hyperfine",
            "--parameter-file",
            "first",
            first_file.path().to_str().unwrap(),
            "-L",
            "middle",
            "a,b",
            "--parameter-file",
            "last",
            last_file.path().to_str().unwrap(),
            "echo {first} {middle} {last}",
        ]);
        let product = product_from_cli_arguments(&matches).unwrap();

        assert_eq!(
            product
                .sources()
                .iter()
                .map(|source| source.name())
                .collect::<Vec<_>>(),
            ["first", "middle", "last"]
        );
        assert_eq!(product.len(), 8);
        assert_eq!(
            product
                .iter()
                .unwrap()
                .take(5)
                .map(|item| text_values(item.unwrap()))
                .collect::<Vec<_>>(),
            [
                [
                    ("first", "x".into()),
                    ("middle", "a".into()),
                    ("last", "q".into())
                ],
                [
                    ("first", "y".into()),
                    ("middle", "a".into()),
                    ("last", "q".into())
                ],
                [
                    ("first", "x".into()),
                    ("middle", "b".into()),
                    ("last", "q".into())
                ],
                [
                    ("first", "y".into()),
                    ("middle", "b".into()),
                    ("last", "q".into())
                ],
                [
                    ("first", "x".into()),
                    ("middle", "a".into()),
                    ("last", "r".into())
                ],
            ]
        );
    }

    #[test]
    fn rejects_duplicate_names_across_source_kinds() {
        let mut file = NamedTempFile::new().unwrap();
        file.write_all(b"x\n").unwrap();
        file.flush().unwrap();

        let matches = get_cli_arguments([
            "hyperfine",
            "-L",
            "value",
            "a,b",
            "--parameter-file",
            "value",
            file.path().to_str().unwrap(),
            "echo {value}",
        ]);
        let error = product_from_cli_arguments(&matches).unwrap_err();

        assert_eq!(error.kind(), std::io::ErrorKind::InvalidInput);
        assert_eq!(error.to_string(), "Duplicate parameter names: value");
    }

    #[test]
    fn reports_a_missing_file_with_path_context() {
        let missing = std::env::temp_dir().join("hyperfine-813-missing-parameter-values.txt");
        let matches = get_cli_arguments([
            "hyperfine",
            "--parameter-file",
            "value",
            missing.to_str().unwrap(),
            "echo {value}",
        ]);
        let error = product_from_cli_arguments(&matches).unwrap_err();

        assert!(error.to_string().contains(missing.to_str().unwrap()));
    }
}

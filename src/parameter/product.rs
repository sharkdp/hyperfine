use std::{io, slice};

use super::{
    file_values::{FileValueLines, FileValues},
    ParameterNameAndValue, ParameterValue,
};

/// A restartable source of text parameter values.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ValuesSource {
    Inline(Vec<String>),
    File(FileValues),
}

impl ValuesSource {
    pub fn inline(values: impl IntoIterator<Item = String>) -> Self {
        Self::Inline(values.into_iter().collect())
    }

    pub fn file(values: FileValues) -> Self {
        Self::File(values)
    }

    pub fn len(&self) -> usize {
        match self {
            Self::Inline(values) => values.len(),
            Self::File(values) => values.len(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    fn iter(&self) -> io::Result<ValuesSourceIter<'_>> {
        match self {
            Self::Inline(values) => Ok(ValuesSourceIter::Inline(values.iter())),
            Self::File(values) => Ok(ValuesSourceIter::File(values.iter()?)),
        }
    }
}

enum ValuesSourceIter<'a> {
    Inline(slice::Iter<'a, String>),
    File(FileValueLines),
}

impl Iterator for ValuesSourceIter<'_> {
    type Item = io::Result<String>;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            Self::Inline(values) => values.next().map(|value| Ok(value.clone())),
            Self::File(values) => values.next(),
        }
    }
}

/// One named dimension in a parameter product.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ParameterSource<'a> {
    name: &'a str,
    values: ValuesSource,
}

impl<'a> ParameterSource<'a> {
    pub fn new(name: &'a str, values: ValuesSource) -> Self {
        Self { name, values }
    }

    pub fn name(&self) -> &'a str {
        self.name
    }

    pub fn values(&self) -> &ValuesSource {
        &self.values
    }
}

/// Restartable Cartesian product of parameter sources.
///
/// The first source changes fastest, matching the ordering used by the current
/// eager `--parameter-list` implementation. File sources retain only their
/// path/count metadata here; an iterator holds just the active line from each
/// source.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ParameterProduct<'a> {
    sources: Vec<ParameterSource<'a>>,
    len: usize,
}

impl<'a> ParameterProduct<'a> {
    pub fn new(sources: Vec<ParameterSource<'a>>) -> io::Result<Self> {
        let len = sources.iter().try_fold(1_usize, |product, source| {
            product.checked_mul(source.values.len()).ok_or_else(|| {
                io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "parameter product is too large",
                )
            })
        })?;

        Ok(Self { sources, len })
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub fn sources(&self) -> &[ParameterSource<'a>] {
        &self.sources
    }

    pub fn iter(&self) -> io::Result<ParameterProductIter<'_, 'a>> {
        let mut iterators = self
            .sources
            .iter()
            .map(|source| source.values.iter())
            .collect::<io::Result<Vec<_>>>()?;

        let mut current = Vec::with_capacity(self.sources.len());
        let mut finished = self.is_empty();
        if !finished {
            for values in &mut iterators {
                match values.next() {
                    Some(Ok(value)) => current.push(value),
                    Some(Err(error)) => return Err(error),
                    None => {
                        finished = true;
                        break;
                    }
                }
            }
        }

        Ok(ParameterProductIter {
            product: self,
            iterators,
            current,
            started: false,
            finished,
        })
    }
}

pub struct ParameterProductIter<'product, 'name> {
    product: &'product ParameterProduct<'name>,
    iterators: Vec<ValuesSourceIter<'product>>,
    current: Vec<String>,
    started: bool,
    finished: bool,
}

impl ParameterProductIter<'_, '_> {
    fn advance(&mut self) -> io::Result<()> {
        for index in 0..self.iterators.len() {
            match self.iterators[index].next() {
                Some(Ok(value)) => {
                    self.current[index] = value;
                    return Ok(());
                }
                Some(Err(error)) => {
                    self.finished = true;
                    return Err(error);
                }
                None => {
                    let mut reset = match self.product.sources[index].values.iter() {
                        Ok(reset) => reset,
                        Err(error) => {
                            self.finished = true;
                            return Err(error);
                        }
                    };
                    match reset.next() {
                        Some(Ok(value)) => {
                            self.current[index] = value;
                            self.iterators[index] = reset;
                        }
                        Some(Err(error)) => {
                            self.finished = true;
                            return Err(error);
                        }
                        None => {
                            self.finished = true;
                            return Ok(());
                        }
                    }
                }
            }
        }

        self.finished = true;
        Ok(())
    }
}

impl<'name> Iterator for ParameterProductIter<'_, 'name> {
    type Item = io::Result<Vec<ParameterNameAndValue<'name>>>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.finished {
            return None;
        }

        if self.started {
            if let Err(error) = self.advance() {
                return Some(Err(error));
            }
            if self.finished {
                return None;
            }
        } else {
            self.started = true;
        }

        Some(Ok(self
            .product
            .sources
            .iter()
            .zip(&self.current)
            .map(|(source, value)| (source.name, ParameterValue::Text(value.clone())))
            .collect()))
    }
}

#[cfg(test)]
mod tests {
    use super::{ParameterProduct, ParameterSource, ValuesSource};
    use crate::parameter::{file_values::FileValues, ParameterValue};
    use std::io::{BufWriter, Write};
    use tempfile::NamedTempFile;

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
    fn preserves_existing_first_dimension_fastest_order() {
        let mut file = NamedTempFile::new().unwrap();
        file.write_all(b"x\ny\n").unwrap();
        file.flush().unwrap();

        let product = ParameterProduct::new(vec![
            ParameterSource::new(
                "inline",
                ValuesSource::inline(["a".to_owned(), "b".to_owned()]),
            ),
            ParameterSource::new(
                "file",
                ValuesSource::file(FileValues::open(file.path()).unwrap()),
            ),
        ])
        .unwrap();

        let combinations = product
            .iter()
            .unwrap()
            .map(|item| text_values(item.unwrap()))
            .collect::<Vec<_>>();

        assert_eq!(product.len(), 4);
        assert_eq!(
            combinations,
            [
                [("inline", "a".into()), ("file", "x".into())],
                [("inline", "b".into()), ("file", "x".into())],
                [("inline", "a".into()), ("file", "y".into())],
                [("inline", "b".into()), ("file", "y".into())],
            ]
        );
    }

    #[test]
    fn empty_source_produces_no_combinations() {
        let product = ParameterProduct::new(vec![ParameterSource::new(
            "empty",
            ValuesSource::inline([]),
        )])
        .unwrap();

        assert!(product.is_empty());
        assert_eq!(product.iter().unwrap().count(), 0);
    }

    #[test]
    fn no_sources_produces_one_empty_combination() {
        let product = ParameterProduct::new(vec![]).unwrap();
        let combinations = product
            .iter()
            .unwrap()
            .collect::<Result<Vec<_>, _>>()
            .unwrap();

        assert_eq!(product.len(), 1);
        assert_eq!(combinations, [vec![]]);
    }

    #[test]
    fn iterates_two_million_file_values_without_materializing_the_product() {
        const LINE_COUNT: usize = 2_000_000;

        let mut file = NamedTempFile::new().unwrap();
        {
            let mut writer = BufWriter::new(file.as_file_mut());
            for index in 0..LINE_COUNT {
                writeln!(writer, "value-{index}").unwrap();
            }
            writer.flush().unwrap();
        }

        let product = ParameterProduct::new(vec![ParameterSource::new(
            "value",
            ValuesSource::file(FileValues::open(file.path()).unwrap()),
        )])
        .unwrap();
        let first = product
            .iter()
            .unwrap()
            .take(3)
            .map(|item| text_values(item.unwrap()))
            .collect::<Vec<_>>();

        assert_eq!(product.len(), LINE_COUNT);
        assert!(matches!(
            product.sources()[0].values(),
            ValuesSource::File(_)
        ));
        assert_eq!(
            first,
            [
                [("value", "value-0".into())],
                [("value", "value-1".into())],
                [("value", "value-2".into())],
            ]
        );
    }
}

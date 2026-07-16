use std::{
    fs::File,
    io::{self, BufRead, BufReader, Lines},
    path::{Path, PathBuf},
};

/// A parameter source backed by lines in a UTF-8 file.
///
/// Construction performs the count pass allowed by issue #813, but retains only
/// the path and line count. Each call to [`FileValues::iter`] opens the file
/// again and yields one owned line at a time.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FileValues {
    path: PathBuf,
    line_count: usize,
}

impl FileValues {
    pub fn open(path: impl Into<PathBuf>) -> io::Result<Self> {
        let path = path.into();
        let line_count = count_lines(&path)?;

        Ok(Self { path, line_count })
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn len(&self) -> usize {
        self.line_count
    }

    pub fn is_empty(&self) -> bool {
        self.line_count == 0
    }

    pub fn iter(&self) -> io::Result<FileValueLines> {
        let file = File::open(&self.path).map_err(|error| path_error(&self.path, "open", error))?;
        let reader = BufReader::new(file);
        Ok(FileValueLines {
            lines: reader.lines(),
            path: self.path.clone(),
            expected_lines: self.line_count,
            yielded_lines: 0,
            finished: false,
        })
    }
}

pub struct FileValueLines {
    lines: Lines<BufReader<File>>,
    path: PathBuf,
    expected_lines: usize,
    yielded_lines: usize,
    finished: bool,
}

impl Iterator for FileValueLines {
    type Item = io::Result<String>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.finished {
            return None;
        }

        match self.lines.next() {
            Some(Ok(line)) if self.yielded_lines < self.expected_lines => {
                self.yielded_lines += 1;
                Some(Ok(line))
            }
            Some(Ok(_)) => {
                self.finished = true;
                Some(Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    format!(
                        "parameter file '{}' contains more lines than during the initial count pass",
                        self.path.display()
                    ),
                )))
            }
            Some(Err(error)) => {
                self.finished = true;
                Some(Err(path_error(&self.path, "read", error)))
            }
            None if self.yielded_lines == self.expected_lines => {
                self.finished = true;
                None
            }
            None => {
                self.finished = true;
                Some(Err(io::Error::new(
                    io::ErrorKind::UnexpectedEof,
                    format!(
                        "parameter file '{}' contains fewer lines than during the initial count pass",
                        self.path.display()
                    ),
                )))
            }
        }
    }
}

fn count_lines(path: &Path) -> io::Result<usize> {
    let file = File::open(path).map_err(|error| path_error(path, "open", error))?;
    BufReader::new(file)
        .lines()
        .try_fold(0_usize, |count, line| {
            line.map_err(|error| path_error(path, "read", error))?;
            count.checked_add(1).ok_or_else(|| {
                io::Error::other(format!(
                    "parameter file '{}' has too many lines",
                    path.display()
                ))
            })
        })
}

fn path_error(path: &Path, action: &str, error: io::Error) -> io::Error {
    io::Error::new(
        error.kind(),
        format!(
            "failed to {action} parameter file '{}': {error}",
            path.display()
        ),
    )
}

#[cfg(test)]
mod tests {
    use super::FileValues;
    use std::{
        fs,
        io::{BufWriter, Write},
    };
    use tempfile::NamedTempFile;

    fn file_with_contents(contents: &[u8]) -> NamedTempFile {
        let mut file = NamedTempFile::new().unwrap();
        file.write_all(contents).unwrap();
        file.flush().unwrap();
        file
    }

    fn values(source: &FileValues) -> Vec<String> {
        source
            .iter()
            .unwrap()
            .collect::<Result<Vec<_>, _>>()
            .unwrap()
    }

    #[test]
    fn supports_lf_crlf_blank_lines_and_a_final_line_without_newline() {
        let lf = file_with_contents(b"alpha\n\nomega");
        let crlf = file_with_contents(b"alpha\r\n\r\nomega");

        let lf_source = FileValues::open(lf.path()).unwrap();
        let crlf_source = FileValues::open(crlf.path()).unwrap();

        assert_eq!(lf_source.len(), 3);
        assert_eq!(crlf_source.len(), 3);
        assert_eq!(values(&lf_source), ["alpha", "", "omega"]);
        assert_eq!(values(&crlf_source), values(&lf_source));
    }

    #[test]
    fn trailing_newline_does_not_create_an_extra_value() {
        let file = file_with_contents(b"alpha\nbeta\n");
        let source = FileValues::open(file.path()).unwrap();

        assert_eq!(source.len(), 2);
        assert_eq!(values(&source), ["alpha", "beta"]);
    }

    #[test]
    fn empty_file_yields_no_values() {
        let file = file_with_contents(b"");
        let source = FileValues::open(file.path()).unwrap();

        assert!(source.is_empty());
        assert!(values(&source).is_empty());
    }

    #[test]
    fn can_reopen_and_iterate_again() {
        let file = file_with_contents(b"one\ntwo\n");
        let source = FileValues::open(file.path()).unwrap();

        assert_eq!(values(&source), ["one", "two"]);
        assert_eq!(values(&source), ["one", "two"]);
    }

    #[test]
    fn rejects_invalid_utf8_during_the_count_pass() {
        let file = file_with_contents(&[b'a', b'\n', 0xff, b'\n']);
        let error = FileValues::open(file.path()).unwrap_err();

        assert_eq!(error.kind(), std::io::ErrorKind::InvalidData);
        assert!(error
            .to_string()
            .contains(&file.path().display().to_string()));
    }

    #[test]
    fn rejects_a_file_truncated_after_the_count_pass() {
        let file = file_with_contents(b"one\ntwo\n");
        let source = FileValues::open(file.path()).unwrap();
        fs::write(file.path(), b"one\n").unwrap();

        let error = source
            .iter()
            .unwrap()
            .collect::<Result<Vec<_>, _>>()
            .unwrap_err();

        assert_eq!(error.kind(), std::io::ErrorKind::UnexpectedEof);
        assert!(error.to_string().contains("fewer lines"));
        assert!(error
            .to_string()
            .contains(&file.path().display().to_string()));
    }

    #[test]
    fn rejects_a_file_extended_after_the_count_pass() {
        let file = file_with_contents(b"one\ntwo\n");
        let source = FileValues::open(file.path()).unwrap();
        fs::write(file.path(), b"one\ntwo\nthree\n").unwrap();

        let error = source
            .iter()
            .unwrap()
            .collect::<Result<Vec<_>, _>>()
            .unwrap_err();

        assert_eq!(error.kind(), std::io::ErrorKind::InvalidData);
        assert!(error.to_string().contains("more lines"));
        assert!(error
            .to_string()
            .contains(&file.path().display().to_string()));
    }

    #[test]
    fn counts_two_million_lines_without_retaining_the_values() {
        const LINE_COUNT: usize = 2_000_000;

        let mut file = NamedTempFile::new().unwrap();
        {
            let mut writer = BufWriter::new(file.as_file_mut());
            for _ in 0..LINE_COUNT {
                writer.write_all(b"value\n").unwrap();
            }
            writer.flush().unwrap();
        }

        let source = FileValues::open(file.path()).unwrap();
        assert_eq!(source.len(), LINE_COUNT);
        assert_eq!(
            source
                .iter()
                .unwrap()
                .take(3)
                .collect::<Result<Vec<_>, _>>()
                .unwrap(),
            ["value", "value", "value"]
        );
    }
}

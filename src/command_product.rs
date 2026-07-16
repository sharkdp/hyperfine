use std::io;

use crate::{
    command::Command,
    parameter::{product::ParameterProduct, ParameterNameAndValue},
};

/// Lazily expands parameterized commands.
///
/// Base commands remain borrowed while each yielded command owns only the
/// current parameter values.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LazyCommandProduct<'a> {
    command_names: Vec<&'a str>,
    command_strings: Vec<&'a str>,
    parameters: ParameterProduct<'a>,
    len: usize,
}

impl<'a> LazyCommandProduct<'a> {
    pub fn new(
        command_names: Vec<&'a str>,
        command_strings: Vec<&'a str>,
        parameters: ParameterProduct<'a>,
    ) -> io::Result<Self> {
        let len = command_strings
            .len()
            .checked_mul(parameters.len())
            .ok_or_else(|| {
                io::Error::new(io::ErrorKind::InvalidInput, "command product is too large")
            })?;

        if command_names.len() > 1 && command_names.len() != len {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!(
                    "expected either one command name or {len}, got {}",
                    command_names.len()
                ),
            ));
        }

        Ok(Self {
            command_names,
            command_strings,
            parameters,
            len,
        })
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub fn iter(&self) -> io::Result<LazyCommandProductIter<'_, 'a>> {
        Ok(LazyCommandProductIter {
            product: self,
            parameters: self.parameters.iter()?,
            current_parameters: None,
            command_index: 0,
            yielded: 0,
        })
    }
}

pub struct LazyCommandProductIter<'product, 'name> {
    product: &'product LazyCommandProduct<'name>,
    parameters: crate::parameter::product::ParameterProductIter<'product, 'name>,
    current_parameters: Option<Vec<ParameterNameAndValue<'name>>>,
    command_index: usize,
    yielded: usize,
}

impl<'name> Iterator for LazyCommandProductIter<'_, 'name> {
    type Item = io::Result<Command<'name>>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.product.command_strings.is_empty() {
            return None;
        }

        if self.current_parameters.is_none() {
            self.current_parameters = match self.parameters.next()? {
                Ok(parameters) => Some(parameters),
                Err(error) => return Some(Err(error)),
            };
            self.command_index = 0;
        }

        let name = self
            .product
            .command_names
            .get(self.yielded)
            .or_else(|| self.product.command_names.first())
            .copied();
        let command = Command::new_parametrized(
            name,
            self.product.command_strings[self.command_index],
            self.current_parameters.as_ref().unwrap().clone(),
        );

        self.command_index += 1;
        self.yielded += 1;
        if self.command_index == self.product.command_strings.len() {
            self.current_parameters = None;
        }

        Some(Ok(command))
    }
}

#[cfg(test)]
mod tests {
    use super::LazyCommandProduct;
    use crate::parameter::{
        file_values::FileValues,
        product::{ParameterProduct, ParameterSource, ValuesSource},
    };
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn base_command_changes_before_the_first_parameter() {
        let mut file = NamedTempFile::new().unwrap();
        file.write_all(b"x\ny\n").unwrap();
        file.flush().unwrap();

        let parameters = ParameterProduct::new(vec![
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
        let commands = LazyCommandProduct::new(
            vec!["benchmark-{inline}-{file}"],
            vec!["echo {inline} {file}", "printf {inline} {file}"],
            parameters,
        )
        .unwrap();
        let lines = commands
            .iter()
            .unwrap()
            .map(|command| command.unwrap().get_command_line())
            .collect::<Vec<_>>();

        assert_eq!(commands.len(), 8);
        assert_eq!(
            lines,
            [
                "echo a x",
                "printf a x",
                "echo b x",
                "printf b x",
                "echo a y",
                "printf a y",
                "echo b y",
                "printf b y",
            ]
        );
    }

    #[test]
    fn supports_one_name_or_one_name_per_generated_command() {
        let parameters = ParameterProduct::new(vec![ParameterSource::new(
            "value",
            ValuesSource::inline(["a".to_owned(), "b".to_owned()]),
        )])
        .unwrap();
        let commands =
            LazyCommandProduct::new(vec!["first", "second"], vec!["echo {value}"], parameters)
                .unwrap();
        let names = commands
            .iter()
            .unwrap()
            .map(|command| command.unwrap().get_name())
            .collect::<Vec<_>>();

        assert_eq!(names, ["first", "second"]);
    }

    #[test]
    fn rejects_an_ambiguous_command_name_count() {
        let parameters = ParameterProduct::new(vec![ParameterSource::new(
            "value",
            ValuesSource::inline(["a".to_owned(), "b".to_owned()]),
        )])
        .unwrap();
        let error = LazyCommandProduct::new(
            vec!["first", "second", "third"],
            vec!["echo {value}"],
            parameters,
        )
        .unwrap_err();

        assert_eq!(error.kind(), std::io::ErrorKind::InvalidInput);
        assert_eq!(
            error.to_string(),
            "expected either one command name or 2, got 3"
        );
    }
}

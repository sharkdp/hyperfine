pub fn tokenize(values: &str) -> Vec<String> {
    let mut tokens = vec![];
    let mut buf = String::new();

    let mut iter = values.chars();
    while let Some(c) = iter.next() {
        match c {
            '\\' => match iter.next() {
                Some(c2 @ ',') | Some(c2 @ '\\') => {
                    buf.push(c2);
                }
                Some(c2) => {
                    buf.push('\\');
                    buf.push(c2);
                }
                None => buf.push('\\'),
            },
            ',' => {
                tokens.push(buf);
                buf = String::new();
            }
            _ => {
                buf.push(c);
            }
        };
    }

    tokens.push(buf);

    tokens
}

#[test]
fn test_tokenize_single_value() {
    assert_eq!(tokenize(r""), vec![""]);
    assert_eq!(tokenize(r"foo"), vec!["foo"]);
    assert_eq!(tokenize(r" "), vec![" "]);
    assert_eq!(tokenize(r"hello\, world!"), vec!["hello, world!"]);
    assert_eq!(tokenize(r"\,"), vec![","]);
    assert_eq!(tokenize(r"\,\,\,"), vec![",,,"]);
    assert_eq!(tokenize(r"\n"), vec![r"\n"]);
    assert_eq!(tokenize(r"\\"), vec![r"\"]);
    assert_eq!(tokenize(r"\\\,"), vec![r"\,"]);
}

#[test]
fn test_tokenize_multiple_values() {
    assert_eq!(tokenize(r"foo,bar,baz"), vec!["foo", "bar", "baz"]);
    assert_eq!(tokenize(r"hello world,foo"), vec!["hello world", "foo"]);

    assert_eq!(tokenize(r"hello\,world!,baz"), vec!["hello,world!", "baz"]);
}

#[test]
fn test_tokenize_empty_values() {
    assert_eq!(tokenize(r"foo,,bar"), vec!["foo", "", "bar"]);
    assert_eq!(tokenize(r",bar"), vec!["", "bar"]);
    assert_eq!(tokenize(r"bar,"), vec!["bar", ""]);
    assert_eq!(tokenize(r",,"), vec!["", "", ""]);
}

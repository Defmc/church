use thiserror::Error;

#[derive(Error, Debug, Clone)]
pub enum Err {
    #[error("this command accepts {1} arguments, but {0} was passed")]
    MissingArgs(/* found */ usize, /* expected */ usize),

    #[error("unknown setting {0:?}")]
    UnknownSetting(String),

    #[error("can't parse {0:?} as a value")]
    ValueParserError(String),

    #[error("unknown command {0:?}")]
    UnknownCommand(String),
}

pub fn get_substr(s: &str) -> Option<&str> {
    let mut iter = s.chars().peekable();
    if iter.next()? != '"' {
        let mut end = 1;
        while let Some(c) = iter.next() {
            match c {
                '"' => return Some(&s[1..end]),
                '\\' if iter.next()? == '"' => end += 2,
                _ => end += 1,
            }
        }
        None
    } else {
        None
    }
}

pub fn get_escape_seq(iter: &mut impl Iterator<Item = char>) -> Option<char> {
    match iter.next()? {
        't' => Some('\t'),
        'n' => Some('\n'),
        'r' => Some('\r'),
        '0' => Some('\0'),
        '"' => Some('\"'),
        '\'' => Some('\''),
        '\\' => Some('\\'),
        'x' => {
            let chars = [iter.next()?, iter.next()?];
            let digits = [chars[0].to_digit(16)?, chars[0].to_digit(16)?];
            let c = digits[0] << 16 | digits[1];
            Some(char::from_u32(c)?)
        }
        _ => None?,
    }
}

pub fn get_args(s: &str) -> Option<Vec<String>> {
    let mut v = Vec::new();
    let mut buf = String::new();
    let mut iter = s.chars();

    #[inline]
    fn collect(v: &mut Vec<String>, buf: &mut String) {
        if !buf.is_empty() {
            v.push(buf.clone());
            buf.clear()
        }
    }

    while let Some(c) = iter.next() {
        match c {
            ' ' => {
                if !buf.is_empty() {
                    collect(&mut v, &mut buf);
                }
            }
            '"' => {
                collect(&mut v, &mut buf);
                loop {
                    match iter.next()? {
                        '\\' => buf.push(get_escape_seq(&mut iter)?),
                        '"' => break,
                        c if true => buf.push(c),
                        _ => unreachable!(),
                    }
                }
                collect(&mut v, &mut buf);
            }
            '\\' => buf.push(get_escape_seq(&mut iter)?),
            _ => buf.push(c),
        }
    }
    collect(&mut v, &mut buf);
    Some(v)
}

#[cfg(test)]
mod tests {
    use super::get_args;

    fn assert_eq_args(s: &str, subs: Option<&[&str]>) {
        assert_eq!(
            get_args(s),
            subs.map(|s| s.into_iter().map(ToString::to_string).collect::<Vec<_>>())
        );
    }

    #[test]
    fn substr() {
        assert_eq_args(
            r#"testing just some "things oops \"""#,
            Some(&["testing", "just", "some", "things oops \""]),
        )
    }

    #[test]
    fn escape() {
        assert_eq_args(
            r#"testing a tab \t now a null \0 and finally the \""#,
            Some(&[
                "testing", "a", "tab", "\t", "now", "a", "null", "\0", "and", "finally", "the",
                "\"",
            ]),
        )
    }

    #[test]
    fn error() {
        assert_eq_args(r#"testing what happens with a ""#, None)
    }
}

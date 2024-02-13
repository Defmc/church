use logos::Logos;

#[derive(Debug, PartialEq, PartialOrd, Clone, Eq, Ord, Logos, Copy)]
pub enum Arg {
    // #[regex(r#""([^\\]|\\.)*""#)]
    #[regex(r#""([^"]|\\.)*""#)]
    StrLit,
    #[regex(r#"[^ ]*"#)]
    Arg,
    #[token("=")]
    Assign,
    #[regex(r"[ \t\n\r]+", logos::skip)]
    Ws,
    #[regex(r#"#.*"#, logos::skip)]
    Comment,
}

impl Arg {
    pub fn parse(s: &impl AsRef<str>) -> impl Iterator<Item = &'_ str> {
        Self::lexer(s.as_ref()).spanned().map(move |(arg, span)| {
            if arg == Ok(Arg::StrLit) {
                &s.as_ref()[span.start + 1..span.end - 1]
            } else {
                &s.as_ref()[span]
            }
        })
    }

    pub fn format(s: &str) -> Option<String> {
        let mut buf = String::with_capacity(s.len());
        let mut it = s.chars();
        while let Some(c) = it.next() {
            if c == '\\' {
                let next = it.next().unwrap();
                let to_push = match next {
                    '0' => '\0',
                    'n' => '\n',
                    'r' => '\r',
                    't' => '\t',
                    '\\' => '\\',
                    '\'' => '\'',
                    '"' => '"',
                    _ => return None,
                };
                buf.push(to_push);
            } else {
                buf.push(c);
            }
        }
        Some(buf)
    }
}

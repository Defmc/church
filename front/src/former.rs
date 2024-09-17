use std::iter::Peekable;

use crate::parser::{ParserToken, Token};

pub fn form<I>(mut it: Peekable<I>) -> Vec<ParserToken>
where
    I: Iterator<Item = ParserToken>,
{
    let mut buf = Vec::new();
    while it.peek().is_some() {
        set_form(&mut it, &mut buf);
    }
    buf
}

fn set_form<I>(it: &mut Peekable<I>, buf: &mut Vec<ParserToken>)
where
    I: Iterator<Item = ParserToken>,
{
    while let Some(tk) = it.next() {
        match tk.1 {
            Token::NewLine if matches!(it.peek(), Some((_, Token::NewLine | Token::Tab, _))) => (),
            Token::NewLine => {
                buf.push(tk);
                break;
            }
            Token::Tab => (),
            Token::Dot => {
                let paren_sp = tk.2;
                buf.push(tk);
                buf.push((paren_sp, Token::OpenParen, paren_sp));
                set_form(it, buf);
                let paren_sp = buf.last().unwrap().2;
                buf.push((paren_sp, Token::CloseParen, paren_sp));
                let blen = buf.len();
                buf.swap(blen - 1, blen - 2);
                break;
            }
            _ => buf.push(tk),
        }
    }
}

use std::iter::Peekable;

use crate::parser::{LexerTy, Token};

pub struct Former<I>
where
    I: Iterator<Item = LexerTy>,
{
    pub it: Peekable<I>,
}

impl<I> Iterator for Former<I>
where
    I: Iterator<Item = LexerTy>,
{
    type Item = LexerTy;
    fn next(&mut self) -> Option<Self::Item> {
        let elm = self.it.next()?;
        match elm.0 {
            Ok(Token::NewLine) => {
                if matches!(self.it.peek(), Some((Ok(Token::Tab | Token::NewLine), _))) {
                    let _ = self.it.next();
                    return self.next();
                }
            }
            Ok(Token::Tab) => return self.next(),
            _ => (),
        }
        Some(elm)
    }
}

impl<I> From<I> for Former<I>
where
    I: Iterator<Item = LexerTy>,
{
    fn from(value: I) -> Self {
        Former {
            it: value.peekable(),
        }
    }
}

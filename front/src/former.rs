use std::iter::Peekable;

use crate::parser::{LexerTy, Token};

pub enum ParenTy {
    Implicit,
    Explicit,
}

pub struct Former<I>
where
    I: Iterator<Item = LexerTy>,
{
    pub it: Peekable<I>,
    pub paren_stack: Vec<ParenTy>,
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
            paren_stack: Vec::new(),
        }
    }
}

use std::iter::Peekable;

use crate::parser::{ParserTokenTy, Token};

pub enum ParenTy {
    Implicit,
    Explicit,
}

pub struct Former<I>
where
    I: Iterator<Item = ParserTokenTy>,
{
    pub it: Peekable<I>,
    pub paren_stack: Vec<ParenTy>,
}

impl<I> Iterator for Former<I>
where
    I: Iterator<Item = ParserTokenTy>,
{
    type Item = ParserTokenTy;
    fn next(&mut self) -> Option<Self::Item> {
        let elm = self.it.next()?;
        match elm.1 {
            Token::NewLine => {
                if matches!(self.it.peek(), Some((_, Token::Tab | Token::NewLine, _))) {
                    return self.next();
                }
            }
            Token::Tab => return self.next(),
            _ => (),
        }
        Some(elm)
    }
}

impl<I> From<I> for Former<I>
where
    I: Iterator<Item = ParserTokenTy>,
{
    fn from(value: I) -> Self {
        Former {
            it: value.peekable(),
            paren_stack: Vec::new(),
        }
    }
}

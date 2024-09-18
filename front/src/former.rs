use std::iter::Peekable;

use crate::parser::{ParserToken, Token};

pub struct Form<I: Iterator<Item = ParserToken>> {
    stack: Vec<Implicit>,
    buf: Vec<ParserToken>,
    it: Peekable<I>,
}

impl<I> Form<I>
where
    I: Iterator<Item = ParserToken>,
{
    pub fn set(&mut self) {
        while let Some(tk) = self.it.next() {
            match tk.1 {
                Token::Tab | Token::NewLine
                    if matches!(self.it.peek(), Some((_, Token::NewLine | Token::Tab, _))) =>
                {
                    continue
                }
                Token::NewLine => self.finish_expr(),
                Token::LetKw => {
                    self.buf.push(tk);
                    self.stack.push(Implicit::Let)
                }
                Token::Comma => {
                    self.goto(Implicit::Let);
                    self.buf.push(tk);
                }
                Token::InKw => {
                    self.finish_let();
                    self.buf.push(tk);
                    self.push_depth(Implicit::In);
                }
                Token::Dot => {
                    self.buf.push(tk);
                    self.push_depth(Implicit::Fn)
                }
                _ => self.buf.push(tk),
            }
        }
        self.finish_expr();
    }

    pub fn finish_let(&mut self) {
        while let Some(pop) = self.stack.pop() {
            if matches!(pop, Implicit::Let) {
                break;
            } else {
                self.push_meta(Token::CloseParen);
            }
        }
    }

    pub fn finish_expr(&mut self) {
        while let Some(_) = self.stack.pop() {
            self.push_meta(Token::CloseParen);
        }
    }

    pub fn push_depth(&mut self, ty: Implicit) {
        self.push_meta(Token::OpenParen);
        self.stack.push(ty);
    }

    pub fn push_meta(&mut self, tk: Token) {
        let last_sp = self.buf.last().unwrap().2;
        self.buf.push((last_sp, tk, last_sp));
    }
}

impl<I> From<I> for Form<I>
where
    I: Iterator<Item = ParserToken>,
{
    fn from(value: I) -> Self {
        Self {
            stack: Vec::default(),
            buf: Vec::default(),
            it: value.peekable(),
        }
    }
}

pub enum Implicit {
    Fn,
    Let,
    In,
}

pub fn form(it: impl Iterator<Item = ParserToken>) -> Vec<ParserToken> {
    let mut form = Form::from(it);
    form.set();
    println!("generated code: {}", Token::rebuild_code(&form.buf));
    form.buf
}

use crate::grammar::ProgramParser;
use crate::parser::{ParserToken, Token};
use crate::compiler::Compiler;
use crate::{Ast, Error};
use logos::Logos;
use std::fs;
use std::path::Path;

pub struct CodeUnit {
    pub scope: Compiler,
    pub program_parser: ProgramParser,
}

impl Default for CodeUnit {
    fn default() -> Self {
        Self {
            scope: Compiler::default(),
            program_parser: ProgramParser::new(),
        }
    }
}

impl CodeUnit {
    pub fn load_file(&mut self, path: impl AsRef<Path>) -> Result<(), Error> {
        let content = fs::read_to_string(path.as_ref())
            .map_err(|_| Error::ModuleNotFound(path.as_ref().into()))?;
        let program = self.parse(&content)?;
        self.eval(program).unwrap();
        Ok(())
    }

    pub fn into_raw_tokens(src: &str) -> Result<Vec<ParserToken>, Error> {
        Token::lexer(src)
            .spanned()
            .try_fold(Vec::new(), |mut ac, (tk, sp)| {
                ac.push((sp.start, tk?, sp.end));
                Ok(ac)
            })
            .map_err(|e| Error::LexerError(e))
    }

    pub fn into_tokens(src: &str) -> Result<impl Iterator<Item = ParserToken>, Error> {
        let tks = Self::into_raw_tokens(src)?.into_iter();
        Ok(crate::former::form(tks.peekable()).into_iter())
    }

    pub fn parse(&mut self, src: impl AsRef<str>) -> Result<Ast, Error> {
        let iter = Self::into_tokens(src.as_ref())?;
        let ast = self
            .program_parser
            .parse(iter)
            .map_err(|e| Error::ParserError(e))?;
        Ok(ast)
    }

    pub fn eval(&mut self, program: Ast) -> Result<(), Error> {
        match program {
            Ast::Program(p) => {
                for atom in p {
                    self.eval(atom)?;
                }
            }
            Ast::Assign(v, m) => {
                let dump = self.scope.dump(&m)?;
                self.scope.insert(v, dump)?;
            }
            Ast::Use(path) => self.load_file(path)?,
            _ => unreachable!(),
        }
        Ok(())
    }
}

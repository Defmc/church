use crate::former::Former;
use crate::grammar::ProgramParser;
use crate::parser::{ParserTokenTy, Token};
use crate::scope::Scope;
use crate::{Ast, Error};
use logos::Logos;
use std::fs;
use std::path::Path;

pub struct CodeUnit {
    pub scope: Scope,
    pub program_parser: ProgramParser,
}

impl Default for CodeUnit {
    fn default() -> Self {
        Self {
            scope: Scope::default(),
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

    pub fn into_raw_tokens(&mut self, src: &str) -> Result<Vec<ParserTokenTy>, Error> {
        Token::lexer(src)
            .spanned()
            .try_fold(Vec::new(), |mut ac, (tk, sp)| {
                ac.push((sp.start, tk?, sp.end));
                Ok(ac)
            })
            .map_err(|e| Error::LexerError(e))
    }

    pub fn into_tokens(&mut self, src: &str) -> Result<impl Iterator<Item = ParserTokenTy>, Error> {
        let tks = self.into_raw_tokens(src)?.into_iter();
        Ok(Former::from(tks))
    }

    pub fn parse(&mut self, src: impl AsRef<str>) -> Result<Ast, Error> {
        let iter = self.into_tokens(src.as_ref())?;
        let ast = self.program_parser.parse(iter).unwrap();
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
        }
        Ok(())
    }
}

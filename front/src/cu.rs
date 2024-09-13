use std::fs;
use std::path::Path;

use logos::Logos;

use crate::former::Former;
use crate::grammar::ProgramParser;
use crate::parser::Token;
use crate::scope::Scope;
use crate::Ast;

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
    pub fn load_file(&mut self, path: impl AsRef<Path>) -> Result<(), ()> {
        let content = fs::read_to_string(path).unwrap();
        let tokens = self.into_iter(&content);
        let program = self.program_parser.parse(tokens).unwrap();
        self.eval(program).unwrap();
        Ok(())
    }

    pub fn into_iter<'a>(
        &mut self,
        src: &'a str,
    ) -> impl Iterator<Item = (usize, Token, usize)> + 'a {
        let lexer = Token::lexer(src).spanned();
        Former::from(lexer).map(|(tk, sp)| (sp.start, tk.unwrap(), sp.end))
    }

    pub fn parse(&mut self, src: impl AsRef<str>) -> Ast {
        let iter = self.into_iter(src.as_ref());
        self.program_parser.parse(iter).unwrap()
    }

    pub fn eval(&mut self, program: Ast) -> Result<(), ()> {
        match program {
            Ast::Program(p) => {
                for atom in p {
                    self.eval(atom).unwrap();
                }
            }
            Ast::Assign(v, m) => {
                let dump = self.scope.dump(&m).unwrap();
                self.scope.insert(v, dump).unwrap();
            }
        }
        Ok(())
    }
}

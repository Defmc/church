use church::Term;
use logos::Logos;

use crate::former::Former;
use crate::grammar::{ProgramAtomParser, ProgramParser};
use crate::parser::Token;
use crate::scope::Scope;
use crate::Ast;

pub struct CodeUnit {
    pub scope: Scope,
    pub program_parser: ProgramParser,
    pub atom_parser: ProgramAtomParser,
}

impl Default for CodeUnit {
    fn default() -> Self {
        Self {
            scope: Scope::default(),
            program_parser: ProgramParser::new(),
            atom_parser: ProgramAtomParser::new(),
        }
    }
}

impl CodeUnit {
    pub fn into_iter<'a>(
        &mut self,
        src: &'a str,
    ) -> impl Iterator<Item = (usize, Token, usize)> + 'a {
        let lexer = Token::lexer(src).spanned();
        Former::from(lexer).map(|(tk, sp)| (sp.start, tk.unwrap(), sp.end))
    }

    pub fn eval(&mut self, program: Ast) -> Result<Option<Term>, ()> {
        match program {
            Ast::Program(p) => {
                for atom in p {
                    self.eval(atom).unwrap();
                }
                Ok(None)
            }
            Ast::Expr(e) => {
                let dumped = self.scope.dump(&e).unwrap();
                Ok(Some(dumped))
            }
            Ast::Assign(v, m) => {
                let dump = self.scope.dump(&m).unwrap();
                self.scope.insert(v, dump).unwrap();
                Ok(None)
            }
        }
    }
}

use logos::Logos;

use crate::former::Former;
use crate::grammar::ProgramParser;
use crate::parser::Token;
use crate::scope::Scope;
use crate::Ast;

pub struct CodeUnit {
    pub scope: Scope,
    pub parser: ProgramParser,
}

impl Default for CodeUnit {
    fn default() -> Self {
        Self {
            scope: Scope::default(),
            parser: ProgramParser::new(),
        }
    }
}

impl CodeUnit {
    pub fn load_from_src(&mut self, src: impl AsRef<str>) -> Result<(), ()> {
        let parsed = self.parse(src.as_ref())?;
        self.load(parsed)
    }

    pub fn parse(&mut self, src: &str) -> Result<Ast, ()> {
        let lexer = Token::lexer(src).spanned();
        let former = Former::from(lexer).map(|(tk, sp)| (sp.start, tk.unwrap(), sp.end));
        let parser = self.parser.parse(former).unwrap();
        Ok(parser)
    }

    pub fn eval(&mut self, program: Ast) -> Result<(), ()> {
        match program {
            Ast::Program(p) => {
                for atom in p {
                    self.eval(atom);
                }
            }
        }
        Ok(())
    }
}

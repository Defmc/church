use std::{fs, path::Path};

use church::Term;

use super::{mode::Mode, scope::Scope, ubody::Dumper, ui::Ui, Ast};

#[derive(Debug, Default)]
pub struct Runner {
    pub scope: Scope,
    pub mode: Mode,
    pub ui: Ui,
}

#[derive(Debug, Clone)]
pub enum Error {
    CantParse,
    InvalidExpr,
    CantLoadFile,
}

impl Runner {
    pub fn run(&mut self, s: &str) -> Result<(), Error> {
        let parsed = super::get_global_parser().parse(s).map_err(|e| {
            eprintln!("parsing error: {e:?}");
            Error::CantParse
        })?;
        let program = parsed.into_program();
        for inst in program {
            match inst {
                Ast::LetExpr(def, imp) => {
                    self.scope.include_from_ubody(&def, &imp);
                }
                Ast::Expr(expr) => {
                    let expr = self.scope.delta_redex(&expr);
                    self.mode.run(&self.ui, &self.scope, expr);
                }
                Ast::Import(path) => self.load(&path)?,
                _ => todo!(),
            }
        }
        Ok(())
    }

    pub fn get_term_from_str(&self, s: &str) -> Result<Term, Error> {
        let parsed = super::get_global_parser().parse(s).map_err(|e| {
            eprintln!("parsing error: {e:?}");
            Error::CantParse
        })?;
        let program = parsed.into_program();
        match program.first().unwrap() {
            Ast::Expr(expr) => {
                let mut dumper = Dumper::new(&self.scope);
                let term = dumper.dump(expr);
                return Ok(term);
            }
            _ => (),
        }
        Err(Error::InvalidExpr)
    }

    pub fn load(&mut self, path: &Path) -> Result<(), Error> {
        match fs::read_to_string(path) {
            Err(e) => {
                eprintln!("error loading file {path:?}: {e:?}");
                Err(Error::CantLoadFile)
            }
            Ok(s) => self.run(&s),
        }
    }
}

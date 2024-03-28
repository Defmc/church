use std::{
    fs,
    path::{Path, PathBuf},
};

use church::Term;

use super::{mode::Mode, scope::Scope, ubody::Dumper, ui::Ui, Ast};

#[derive(Debug, Default)]
pub struct Runner {
    pub scope: Scope,
    pub mode: Mode,
    pub loaded_files: Vec<PathBuf>,
    pub ui: Ui,
}

#[derive(Debug, Clone)]
pub enum Error {
    CantParse,
    InvalidExpr,
    CantLoadFile,
    InvalidReference,
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
                    let expr = self
                        .scope
                        .delta_redex(&expr)
                        .ok_or(Error::InvalidReference)?;
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
        if let Ast::Expr(ref expr) = program[0] {
            let mut dumper = Dumper::new(&self.scope);
            let term = dumper.dump(expr).ok_or(Error::InvalidReference)?;
            return Ok(term);
        }
        Err(Error::InvalidExpr)
    }

    pub fn load(&mut self, path: &Path) -> Result<(), Error> {
        let path = path.into();
        if self.loaded_files.contains(&path) {
            return Ok(());
        }
        match fs::read_to_string(&path) {
            Err(e) => {
                eprintln!("error loading file {path:?}: {e:?}");
                Err(Error::CantLoadFile)
            }
            Ok(s) => {
                self.loaded_files.push(path);
                self.run(&s)
            }
        }
    }
}

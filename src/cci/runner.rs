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
}

impl Runner {
    pub fn run(&mut self, s: &str) -> Result<(), Error> {
        let parsed = super::get_global_parser()
            .parse(s)
            .map_err(|_| Error::CantParse)?;
        let program = parsed.into_program();
        for inst in program {
            match *inst {
                Ast::LetExpr(def, imp) => {
                    self.scope.include_from_ubody(&def, imp.as_ref());
                }
                Ast::Expr(expr) => {
                    let expr = self.scope.delta_redex(expr.as_ref());
                    self.mode.run(&self.ui, &self.scope, expr);
                }
                _ => unreachable!(),
            }
        }
        Ok(())
    }

    pub fn get_term_from_str(&self, s: &str) -> Result<Term, Error> {
        let parsed = super::get_global_parser()
            .parse(s)
            .map_err(|_| Error::CantParse)?;
        let program = parsed.into_program();
        for inst in program {
            match *inst {
                Ast::LetExpr(..) => {
                    unreachable!()
                }
                Ast::Expr(expr) => {
                    let mut dumper = Dumper::new(&self.scope);
                    let term = dumper.dump(&expr);
                    return Ok(term);
                }
                _ => unreachable!(),
            }
        }
        Err(Error::InvalidExpr)
    }
}

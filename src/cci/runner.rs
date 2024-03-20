use super::{scope::Scope, ubody::Dumper, Ast};

#[derive(Debug, Default)]
pub struct Runner {
    pub scope: Scope,
}

#[derive(Debug, Clone)]
pub enum Error {
    CantParse,
}

impl Runner {
    pub fn run(&mut self, s: &str) -> Result<(), Error> {
        let parsed = super::get_global_parser()
            .parse(s)
            .map_err(|_| Error::CantParse)?;
        match parsed.as_ref() {
            Ast::LetExpr(def, imp) => {
                self.scope
                    .include(def.to_string(), imp.delta_redex(&self.scope));
            }
            Ast::Expr(expr) => {
                let mut dumper = Dumper::new(&self.scope);
                println!("dump: {}", dumper.dump(&expr));
            }
        }
        Ok(())
    }
}

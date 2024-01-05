use std::fs::File;
use std::io::{BufWriter, Write};
use std::str::FromStr;

use church::scope::Scope;
use church::Body;
use rustyline::config::Configurer;
use rustyline::error::ReadlineError;
use rustyline::DefaultEditor;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    bootstrap()?;
    #[cfg(feature = "repl")]
    repl()?;
    Ok(())
}

fn repl() -> Result<(), Box<dyn std::error::Error>> {
    let mut last_expr = church::Body::id();
    let mut rl = DefaultEditor::new()?;
    rl.set_auto_add_history(true);
    rl.set_history_ignore_space(true);
    let mut scope = Scope::default();
    loop {
        let buf = match rl.readline("λ> ") {
            Ok(s) => s,
            Err(e) => {
                return if matches!(e, ReadlineError::Eof) {
                    Ok(())
                } else {
                    Err(Box::new(e))
                };
            }
        };
        run(buf, &mut scope, &mut last_expr);
    }
}

fn run(mut expr: String, scope: &mut Scope, last: &mut Body) {
    let new_scope = Scope::from_str(&expr).unwrap();
    if new_scope.defs.is_empty() {
        let origin = expr.clone();
        let delta = scope.delta_redex(&mut expr);
        let lex = church::parser::lexer(&expr);
        match church::parser::parse(lex) {
            Ok(expr) => {
                println!("expr: {origin}");
                println!("\tδ-eq:    {}", !delta);
                println!("\tδ-redex: {expr}");
                println!("\tα-eq:    {}", last.alpha_eq(&expr));
                println!("\tα-redex: {}", expr.clone().alpha_reduced());
                println!("\t\t-> β:  {}", expr.clone().alpha_reduced().beta_reduced());
                println!("\tβ-redex: {}", expr.clone().beta_reduced());
                println!("\t\t-> α:  {}", expr.clone().beta_reduced().alpha_reduced());
                *last = expr;
            }
            Err(e) => println!("\terror:   {e:?}"),
        }
    } else {
        scope.extend(new_scope);
    }
}

fn bootstrap() -> Result<(), Box<dyn std::error::Error>> {
    let src = include_str!("parser/parser.wop");
    let mut builder = wop::builder::Builder::from_str(src).unwrap();
    builder.entry_type = "Gramem".to_string();
    let file = File::create("src/parser/out.rs").unwrap();
    let mut writer = BufWriter::new(file);
    writeln!(
        writer,
        r#"
use super::{{Ast, Gramem, Meta, Sym}};
use lrp::Grammar;
use lrp::ReductMap;
#[allow({})]
#[must_use]
pub fn grammar() -> Grammar<Sym> {{
    Grammar::new(Sym::EntryPoint, {}, Sym::Eof)
}}"#,
        wop::builder::GRAMMAR_LINTS,
        builder.dump_grammar(src)
    )?;

    writeln!(
        writer,
        r#"
#[allow({})]
pub fn reduct_map() -> ReductMap<Meta<Ast>, Sym> {}"#,
        wop::builder::REDUCTOR_LINTS,
        builder.dump_reductor(src),
    )?;
    Ok(())
}

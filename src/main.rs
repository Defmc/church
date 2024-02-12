use std::fs::File;
use std::io::{BufWriter, Write};
use std::str::FromStr;

#[cfg(feature = "repl")]
pub mod repl;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    bootstrap()?;
    #[cfg(feature = "repl")]
    repl()?;
    Ok(())
}

#[cfg(feature = "repl")]
fn repl() -> Result<(), Box<dyn std::error::Error>> {
    use repl::Repl;

    let mut repl = Repl::default();
    repl.start()
}

// fn info(mut expr: String, scope: &mut Scope, last: &mut Body) {
//     let new_scope = Scope::from_str(&expr).unwrap();
//     if new_scope.defs.is_empty() {
//         let origin = expr.clone();
//         let delta = scope.delta_redex(&mut expr);
//         println!("expr: {origin}");
//         println!("\tδ-eq:    {}", !delta);
//         println!("\tδ-redex: {expr}");
//         let lex = church::parser::lexer(&expr);
//         match church::parser::parse(lex) {
//             Ok(expr) => {
//                 println!(
//                     "\tδ-match: {}",
//                     scope.get_from_alpha_key(&expr).unwrap_or("n/a")
//                 );
//                 println!("\tα-eq:    {}", last.alpha_eq(&expr));
//                 println!("\tα-redex: {}", expr.clone().alpha_reduced());
//                 println!(
//                     "\t   -> β:  {}",
//                     expr.clone().alpha_reduced().beta_reduced()
//                 );
//                 println!("\tβ-redex: {}", expr.clone().beta_reduced());
//                 println!(
//                     "\t   -> α:  {}",
//                     expr.clone().beta_reduced().alpha_reduced()
//                 );
//                 println!(
//                     "\tβ-match: {}",
//                     scope
//                         .get_from_alpha_key(&expr.clone().beta_reduced())
//                         .unwrap_or("n/a")
//                 );
//                 // TODO: Match system
//                 *last = expr;
//             }
//             Err(e) => println!("\terror:   {e:?}"),
//         }
//     } else {
//         scope.extend(new_scope);
//     }
// }

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

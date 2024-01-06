use std::fs::File;
use std::io::{BufWriter, Write};
use std::str::FromStr;

fn main() -> Result<(), Box<dyn std::error::Error>> {
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
    drop(writer);
    let mut buf = String::new();
    loop {
        print!("\n\\> ");
        std::io::stdout().flush().unwrap();
        buf.clear();
        std::io::stdin().read_line(&mut buf)?;
        let lex = church::parser::lexer(&buf);
        match church::parser::parse(lex) {
            Ok(expr) => {
                println!("\texpr:    {expr}");
                println!("\tα-redex: {}", expr.clone().alpha_reduced());
                println!("\t\t-> β:  {}", expr.clone().alpha_reduced().beta_reduced());
                println!("\tβ-redex: {}", expr.clone().beta_reduced());
                println!("\t\t-> α:  {}", expr.clone().beta_reduced().alpha_reduced());
            }
            Err(e) => println!("\terror:   {e:?}"),
        }
    }
}

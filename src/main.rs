use church::parser;
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
        builder.dump_grammar(&src)
    )?;

    writeln!(
        writer,
        r#"
#[allow({})]
pub fn reduct_map() -> ReductMap<Meta<Ast>, Sym> {}"#,
        wop::builder::REDUCTOR_LINTS,
        builder.dump_reductor(&src),
    )?;
    println!("samples");
    println!("input\t\toutput");
    const SAMPLES: &[&str] = &["^a.a", "^a^b.a", "^a^b.b"];
    for sample in SAMPLES {
        let lex = parser::lexer(sample);
        let mut parser = parser::parser(lex);
        let err = parser.start();
        assert!(err.is_ok(), "{err:?}");
        let out = parser.items[0].item.item.as_expr();
        println!("{sample}{out}");
    }

    Ok(())
}

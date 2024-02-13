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

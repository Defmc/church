#[cfg(feature = "repl")]
pub mod repl;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    repl()?;
    Ok(())
}

#[cfg(feature = "repl")]
fn repl() -> Result<(), Box<dyn std::error::Error>> {
    use repl::Repl;

    let mut repl = Repl::default();
    repl.start()
}

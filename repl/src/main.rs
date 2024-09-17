fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;
    let mut repl = repl::Repl::default();
    repl.run()
}

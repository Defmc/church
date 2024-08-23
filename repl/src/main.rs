fn main() -> color_eyre::Result<()> {
    let mut repl = repl::Repl::default();
    repl.run()
}

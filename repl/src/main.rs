use std::env;

fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;
    let mut repl = repl::Repl::default();

    let mut args = env::args().skip(1).peekable();
    if let Some(next) = args.peek() {
        if let Ok(meta) = std::fs::metadata(next) {
            if meta.is_file() {
                todo!("run binary modules")
            } else {
                panic!("the binary module should be a standalone file");
            }
        } else {
            for arg in args {
                repl.handle(&arg);
            }
            Ok(())
        }
    } else {
        repl.run()
    }
}

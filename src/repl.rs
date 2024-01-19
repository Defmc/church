use church::{scope::Scope, Body};
use rustyline::{config::Configurer, error::ReadlineError, DefaultEditor};
use std::path::PathBuf;

pub type Result = std::result::Result<(), Box<dyn std::error::Error>>;

pub type Handler = fn(&mut Repl, &str);

pub const HANDLERS: &[(&str, Handler)] = &[];

#[derive(Debug)]
pub struct Repl {
    scope: Scope,
    last_expr: Body,
    loaded_files: Vec<PathBuf>,
    prompt: String,
    rl: DefaultEditor,
}

impl Default for Repl {
    fn default() -> Self {
        let mut rl = DefaultEditor::new().unwrap();
        rl.set_auto_add_history(true);
        rl.set_history_ignore_space(true);
        Repl {
            scope: Scope::default(),
            last_expr: Body::id(),
            loaded_files: Vec::default(),
            prompt: String::from("λ> "),
            rl,
        }
    }
}

impl Repl {
    pub fn start(&mut self) -> Result {
        loop {
            let buf = match self.rl.readline(&self.prompt) {
                Ok(s) => s,
                Err(e) => {
                    return if matches!(e, ReadlineError::Eof) {
                        Ok(())
                    } else {
                        Err(Box::new(e))
                    };
                }
            };
            self.handle(buf.trim());
        }
    }

    pub fn handle(&mut self, input: &str) {
        for (prefix, h) in HANDLERS.iter() {
            if input.starts_with(prefix) {
                let stripped = input.strip_prefix(prefix).unwrap();
                h(self, stripped)
            }
        }
    }
}

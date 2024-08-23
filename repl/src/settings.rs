pub struct Settings {
    pub prompt: String,
    pub show_tokens: bool,
    pub show_ast: bool,
    pub run: bool,
    pub bench: bool,
    pub show_output: bool,
    pub prettify: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            prompt: "λ> ".into(),
            show_tokens: false,
            show_ast: false,
            run: true,
            bench: false,
            show_output: true,
            prettify: true,
        }
    }
}

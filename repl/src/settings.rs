use std::str::FromStr;

pub struct Settings {
    pub prompt: String,
    pub show_tokens: bool,
    pub show_ast: bool,
    pub show_form: bool,
    pub run: bool,
    pub bench: bool,
    pub show_output: bool,
    pub prettify: bool,
    pub b_order: BetaOrder,
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
            show_form: false,
            b_order: BetaOrder::default(),
        }
    }
}

#[derive(Default)]
pub enum BetaOrder {
    #[default]
    Normal,
    CallByValue,
}

impl FromStr for BetaOrder {
    type Err = crate::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "normal" => Ok(Self::Normal),
            "call-by-value" => Ok(Self::CallByValue),
            _ => Err(crate::Error::UnknownBetaOrder),
        }
    }
}

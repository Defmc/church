use church::Term;
use rustc_hash::FxHashSet as HashSet;

use super::{env, io, proc, Repl};

pub const COMMANDS: &[Command] = &[
    Command {
        name: "quit",
        help: "quits the repl",
        inputs_help: &[],
        handler: env::quit_fn,
    },
    Command {
        name: "show",
        help: "shows something from the repl",
        inputs_help: &[("<thing>", "thing to be shown, including:\n\t\t\tscope: all the aliases and expressions defined by the user\n\t\t\tenv: the repl environment\n\t\t\tloaded: all the loaded files as the filepaths\n\t\t\t<alias>: the expression from the scope")],
        handler: env::show_fn,
    },
    Command {
        name: "help",
        help: "show the help message of something",
        inputs_help: &[("<thing>", "thing to be shown")],
        handler: env::help_fn,
    },
    Command {
        name: "set",
        help: "configure something from the repl",
        inputs_help: &[
            (
                "readable <true or false>",
                "sets if the repl should format lambda expressions (default = true)",
            ),
            (
                "prompt <str>",
                "sets the prompt message before each repl line (default = \"λ> \")",
            ),
            ("mode <mode>", "sets the repl mode:\n\t\t\tnormal: it's the default\n\t\t\tdebug: allows you to check and stop every line\n\t\t\tvisual: shows you each step of beta reductions\n\t\t\tbench: benchmarks everything executed"),
            ("history <true or false>", "enable or disable addition to history (default = true)" )
        ],
        handler: env::set,
    },
    Command {
        name: "delta",
        help: "delta reduces the expression"
        ,inputs_help: &[("<expr>", "the lambda expression")],
        handler: proc::delta
    },
    Command {
        name: "assert_eq",
        help: "asserts equality between two lambda expressions. If they're different, panics.",
            inputs_help: &[("<lhs-expr> <rhs-expr>", "the lambda expressions to be compared"), ("-q", "quiet mode: just runs and crashes if needed. Don't should anything more than the panic message")],
            handler: env::assert_eq
    },
    Command {
        name: "load",
        help: "loads a file inside repl",
        inputs_help: &[("<filepath>", "file to be loaded"), ("-s", "strictly loads the file. Updating the lazy-evaluation system immediately")],
        handler: io::load
    },
    Command {
        name: "alpha_eq",
        help: "check if two lambda expressions are alpha equivalent",
            inputs_help: &[("<lhs-expr> <rhs-expr>", "the lambda expressions to be compared")],
            handler: proc::alpha_eq
    }
    ,Command {
        name: "alpha",
        help: "alpha reduces the lambda expression",
        inputs_help: &[("<expr>", "the expression to be reduced")],
        handler: proc::alpha
    },
    Command {
        name: "closed",
        help: "show the term's closedness structure",
        inputs_help: &[("<expr>", "the expression to be analyzed")],
        handler: proc::closed
    },
    Command {
        name: "debrejin",
        help: "debrejin-alpha reduces the lambda expression",
        inputs_help: &[("<expr>", "the expression to be reduced")],
        handler: proc::debrejin
    },
    Command {
        name: "gen_nats",
        help: "binds the natural numbers of the interval",
        inputs_help: &[("<number from> <number to>", "the range of numbers to be binded")],
        handler: env::gen_nats
    },
    Command {
        name: "fix_point",
        help: "solves the recursion of an expression, using the Y combinator",
        inputs_help: &[("<expr>", "the expression to be fixed")],
        handler: proc::fix_point
    },
    Command {
        name: "len",
        help: "get the length of an expression - how many elements/symbols there are",
        inputs_help: &[("<expr>", "the expression to get length")],
        handler: proc::len
    },
    Command {
        name: "straight",
        help: "straightforward beta redex an expression",
        inputs_help: &[("<expr>", "the expression to reduce")],
        handler: proc::straight
    }
];

pub struct Command {
    pub name: &'static str,
    pub help: &'static str,
    pub inputs_help: &'static [(&'static str, &'static str)],
    pub handler: fn(CmdEntry),
}

pub struct CmdEntry<'a> {
    pub inputs: Vec<&'a str>,
    pub flags: HashSet<&'a str>,
    pub repl: &'a mut Repl,
}

impl<'a> CmdEntry<'a> {
    pub fn into_expr(&mut self) -> std::result::Result<Term, crate::cci::runner::Error> {
        let input = self.inputs.join(" ");
        self.repl.runner.get_term_from_str(&input)
    }
}

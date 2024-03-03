use church::Term;
use std::{
    io::Write,
    str::FromStr,
    sync::atomic::{AtomicBool, Ordering},
    time::Instant,
};

use super::{scope::Scope, ui::Ui};
pub static INTERRUPT: AtomicBool = AtomicBool::new(false);

#[derive(Default, Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Mode {
    Debug,
    Visual,
    VisualTrace,
    Trace,
    Bench,
    #[default]
    Normal,
}

impl Mode {
    pub fn should_show(&self) -> bool {
        match self {
            Self::Visual | Self::Debug | Self::VisualTrace => true,
            _ => false,
        }
    }

    pub fn bench(&self, op: &str, f: impl FnOnce()) {
        let start = Instant::now();
        f();
        if self == &Self::Bench {
            println!("[{op}: {:?}]", start.elapsed())
        };
    }

    pub fn run(&self, ui: &Ui, scope: &Scope, mut expr: Term) {
        let mut steps = 0;
        let mut start = Instant::now();
        expr.update_closed();
        while expr.beta_redex_step() && !INTERRUPT.load(Ordering::Acquire) {
            let elapsed_beta_time = start.elapsed();
            steps += 1;
            if self.should_show() {
                ui.print(scope, &expr);
            }
            if self == &Self::Trace {
                println!(
                    "step {steps} | len: {} | time: {elapsed_beta_time:?}",
                    expr.len()
                );
            }
            if self == &Self::Debug {
                Self::wait_response(steps);
            }
            start = Instant::now();
        }
        let _ = INTERRUPT.compare_exchange(true, false, Ordering::Acquire, Ordering::Relaxed);
        if !self.should_show() {
            ui.print(scope, &expr);
        }
    }

    pub fn wait_response(steps: usize) {
        let mut buf = String::new();
        loop {
            print!("[step {steps}] (c)ontinue or (a)bort: ");
            std::io::stdout().flush().unwrap();
            buf.clear();
            std::io::stdin().read_line(&mut buf).unwrap();
            match buf.trim() {
                "c" => break,
                "a" => return,
                "" => break,
                _ => eprintln!("unknown option"),
            }
        }
    }
}

impl FromStr for Mode {
    type Err = ();
    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s {
            "debug" => Ok(Self::Debug),
            "visual" => Ok(Self::Visual),
            "visualtrace" => Ok(Self::VisualTrace),
            "normal" => Ok(Self::Normal),
            "bench" => Ok(Self::Bench),
            "trace" => Ok(Self::Trace),
            _ => Err(()),
        }
    }
}

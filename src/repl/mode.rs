use super::Repl;
use church::Term;
use std::{io::Write, str::FromStr, sync::atomic::Ordering, time::Instant};

#[derive(Default, Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Mode {
    Debug,
    Visual,
    Trace,
    Bench,
    #[default]
    Normal,
}

impl Mode {
    pub fn should_show(&self, repl: &Repl) -> bool {
        match self {
            Self::Visual | Self::Debug => true,
            Self::Trace => repl.visual_trace,
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

    pub fn run(&self, repl: &mut Repl, mut l: String) {
        self.bench("delta redex", || {
            l = repl.scope.delta_redex(&l).0;
        });
        let mut expr = match Term::try_from_str(&l) {
            Ok(e) => e,
            Err(e) => {
                eprintln!("error: {e:?}");
                return;
            }
        };
        let mut steps = 0;
        let mut start = Instant::now();
        expr.update_closed();
        while expr.beta_redex_step() && !super::INTERRUPT.load(Ordering::Acquire) {
            let elapsed_beta_time = start.elapsed();
            steps += 1;
            if self.should_show(repl) {
                println!("{}", repl.format_value(&expr));
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
        let _ =
            super::INTERRUPT.compare_exchange(true, false, Ordering::Acquire, Ordering::Relaxed);
        if !self.should_show(repl) {
            println!("{}", repl.format_value(&expr));
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
            "normal" => Ok(Self::Normal),
            "bench" => Ok(Self::Bench),
            "trace" => Ok(Self::Trace),
            _ => Err(()),
        }
    }
}

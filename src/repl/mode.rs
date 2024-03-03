use super::Repl;
use church::Term;
use std::{io::Write, str::FromStr, time::Instant};

#[derive(Default, Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Mode {
    Debug,
    Visual,
    Bench,
    #[default]
    Normal,
}

impl Mode {
    pub fn should_show(&self) -> bool {
        self == &Self::Visual || self == &Self::Debug
    }

    pub fn bench(&self, op: &str, f: impl FnOnce()) {
        let start = Instant::now();
        f();
        if self == &Self::Bench {
            println!("[{op}: {:?}]", start.elapsed())
        };
    }

    pub fn run_show(&self, repl: &mut Repl, l: &mut Term) {
        let mut buf = String::new();
        println!("{l}");
        let mut steps = 0;
        let mut start = Instant::now();
        l.update_closed();
        while l.beta_redex_step() {
            println!("{}", repl.format_value(l));
            let elapsed_beta_time = start.elapsed();
            steps += 1;
            println!(
                "step {steps} | len: {} | time: {elapsed_beta_time:?}",
                l.len()
            );
            if self == &Self::Debug {
                loop {
                    print!("[step {steps}] (c)ontinue or (a)bort: ");
                    assert!(std::io::stdout().flush().is_ok());
                    buf.clear();
                    assert!(std::io::stdin().read_line(&mut buf).is_ok());
                    match buf.trim() {
                        "c" => break,
                        "a" => return,
                        "" => break,
                        _ => eprintln!("unknown option"),
                    }
                }
            }
            start = Instant::now();
        }
    }

    pub fn run(&self, repl: &mut Repl, mut l: String) {
        self.bench("delta redex", || {
            l = repl.scope.delta_redex(&l).0;
        });
        let l = if self.should_show() {
            match Term::try_from_str(&l) {
                Ok(mut l) => {
                    self.bench("beta redex", || self.run_show(repl, &mut l));
                    l
                }
                Err(e) => {
                    eprintln!("error: {e:?}");
                    return;
                }
            }
        } else {
            match Term::try_from_str(&l) {
                Ok(mut l) => {
                    self.bench("beta redex", || {
                        l.beta_redex();
                    });
                    l
                }
                Err(e) => {
                    eprintln!("error: {e:?}");
                    return;
                }
            }
        };
        self.bench("printing", || {
            repl.print_value(&l);
        });
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
            _ => Err(()),
        }
    }
}

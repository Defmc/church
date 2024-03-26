use super::{cmds::COMMANDS, parser::Arg, CmdEntry};
use church::{Body, Term};
use core::fmt;
use rustyline::config::Configurer;
use std::{io::Write, rc::Rc, str::FromStr};

pub fn set(e: CmdEntry) {
    fn set_with<T: FromStr>(opt: &mut T, s: &str)
    where
        <T as FromStr>::Err: fmt::Debug,
    {
        match T::from_str(s) {
            Ok(v) => *opt = v,
            Err(e) => println!("unknown value {:?}: {e:?}", s),
        }
    }

    match e.inputs.len() {
        0 => {
            eprintln!("missing option and value");
            return;
        }
        1 => {
            eprintln!("missing value");
            return;
        }
        _ => (),
    };
    match e.inputs[0] {
        "readable" => set_with(&mut e.repl.readable, e.inputs[1]),
        "binary_numbers" => set_with(&mut e.repl.binary_numbers, e.inputs[1]),
        "prompt" => match Arg::format(e.inputs[1]) {
            Some(v) => set_with(&mut e.repl.prompt, &v),
            None => eprintln!("bad format string {:?}", e.inputs[1]),
        },
        "mode" => set_with(&mut e.repl.runner.mode, e.inputs[1]),
        "history" => match bool::from_str(e.inputs[1]) {
            Ok(opt) => e.repl.rl.set_auto_add_history(opt),
            Err(err) => eprintln!("unknown value {:?}: {err:?}", e.inputs[1]),
        },
        _ => eprintln!("unknonwn option {:?}", e.inputs[0]),
    }
}

pub fn help_fn(e: CmdEntry) {
    if e.inputs.is_empty() {
        eprintln!(
            "missing a command to see help. Type `show commands` to see the available commands."
        );
        return;
    }
    for hs in COMMANDS.iter() {
        if e.inputs[0] == hs.name {
            println!("{}", hs.name);
            println!("\tdescription:");
            println!("\t\t{}", hs.help);
            println!("\tinputs:");
            for (i, h) in hs.inputs_help {
                println!("\t\t{i}: {h}");
            }
        }
    }
}

pub fn num_to_church(n: usize) -> Term {
    fn natural_body(n: usize) -> Term {
        let body = if n == 0 {
            Body::Id(1)
        } else {
            Body::App(Term::new(Body::Id(0)), natural_body(n - 1))
        };
        Term::new(body)
    }
    natural_body(n).with([0, 1])
}

pub fn num_to_bin_list(n: usize) -> String {
    fn natural_body(n: usize) -> String {
        match n {
            0 => "(Cons False Nil)".into(),
            _ => format!(
                "(Cons {} {})",
                if n & 0b1 == 1 { "True" } else { "False" },
                natural_body(n >> 1)
            ),
        }
    }
    natural_body(n)
}

pub fn gen_nats(e: CmdEntry) {
    let start = if let Ok(s) = usize::from_str(e.inputs[0]) {
        s
    } else {
        println!("{:?} is not a valid range start", e.inputs[0]);
        return;
    };
    let end = if let Ok(e) = usize::from_str(e.inputs[1]) {
        e
    } else {
        println!("{:?} is not a valid range end", e.inputs[1]);
        return;
    };
    for i in start..=end {
        let imp = if e.repl.binary_numbers {
            num_to_bin_list(i)
        } else {
            num_to_church(i).to_string()
        };
        e.repl.runner.scope.include(
            &i.to_string(),
            e.repl.runner.get_term_from_str(&imp).unwrap(),
        );
    }
}

pub fn quit_fn(e: CmdEntry) {
    e.repl.quit = true;
}

pub fn show_fn(e: CmdEntry) {
    if e.inputs.is_empty() {
        eprintln!("missing something to be shown");
        return;
    }
    match e.inputs[0] {
        "scope" => {
            for (d, i) in e.repl.runner.scope.definitions.iter() {
                println!("{d} = {i}");
            }
        }
        "env" => {
            println!("{:?}", e.repl);
        }
        "loaded" => {
            for p in e.repl.loaded_files.iter() {
                println!("{p:?}");
            }
        }
        "commands" => {
            COMMANDS.iter().for_each(|c| println!("{}", c.name));
        }
        _ => {
            if let Some(def) = e.repl.runner.scope.definitions.get(e.inputs[0]) {
                println!("{def}");
            } else {
                eprintln!("unknown option {:?}", e.inputs[0]);
            }
        }
    }
}

pub fn assert_eq(mut e: CmdEntry) {
    match e.into_expr() {
        Ok(mut expr) => match Rc::make_mut(&mut expr.body) {
            Body::App(ref mut lhs, ref mut rhs) => {
                let (lhs_s, rhs_s) = (e.repl.format_value(lhs), e.repl.format_value(rhs));
                if !e.flags.contains("q") {
                    print!("testing {lhs_s} == {rhs_s}... ",);
                }
                std::io::stdout().flush().unwrap();
                lhs.beta_redex();
                rhs.beta_redex();
                if !lhs.alpha_eq(rhs) {
                    eprintln!("\nerror: they're different");
                    eprintln!("\t{lhs_s} -> {}", e.repl.format_value(lhs));
                    eprintln!("\t{rhs_s} -> {}", e.repl.format_value(rhs));
                    panic!("assertion failed");
                }
                println!("ok!");
            }
            _ => eprintln!("error: missing another expression"),
        },
        Err(e) => panic!("error: {e:?}"),
    }
}

use super::{Arg, CmdEntry, COMMANDS};
use church::{scope::Scope, Body, Term};
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
        "prompt" => match Arg::format(e.inputs[1]) {
            Some(v) => set_with(&mut e.repl.prompt, &v),
            None => eprintln!("bad format string {:?}", e.inputs[1]),
        },
        "mode" => set_with(&mut e.repl.mode, e.inputs[1]),
        "history" => match bool::from_str(e.inputs[1]) {
            Ok(opt) => e.repl.rl.set_auto_add_history(opt),
            Err(err) => eprintln!("unknown value {:?}: {err:?}", e.inputs[1]),
        },
        _ => eprintln!("unknonwn option {:?}", e.inputs[0]),
    }
}

pub fn help_fn(e: CmdEntry) {
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

pub fn gen_nats(e: CmdEntry) {
    fn natural(n: usize) -> Term {
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
    let mut numbers = Scope::default();
    for i in start..end {
        numbers.aliases.push(i.to_string());
        numbers.defs.push(natural(i).to_string());
    }
    numbers.update();
    e.repl.scope.extend(numbers);
}

pub fn quit_fn(e: CmdEntry) {
    e.repl.quit = true;
}

pub fn show_fn(e: CmdEntry) {
    match e.inputs[0] {
        "scope" => {
            for (k, v) in e.repl.scope.aliases.iter().zip(e.repl.scope.defs.iter()) {
                println!("{k} = {v}");
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
        _ => {
            if let Some(def) = e.repl.scope.indexes.get(e.inputs[0]) {
                println!("{}", e.repl.scope.defs[*def]);
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
                print!("testing {lhs_s} == {rhs_s}... ",);
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
        Err(e) => eprintln!("error: {e:?}"),
    }
}

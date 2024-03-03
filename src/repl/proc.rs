use std::str::FromStr;

use super::{CmdEntry, Repl};
use church::{scope::Scope, straight::StraightRedex, Body};

pub fn delta(mut e: CmdEntry) {
    match e.into_expr() {
        Ok(expr) => {
            println!("{}", expr);
        }
        Err(e) => eprintln!("error: {e:?}"),
    }
}

pub fn alpha_eq(mut e: CmdEntry) {
    match e.into_expr() {
        Ok(expr) => match expr.body.as_ref() {
            Body::App(ref lhs, ref rhs) => {
                println!("{}", lhs.alpha_eq(rhs));
            }
            _ => eprintln!("missing the second expression"),
        },
        Err(e) => eprintln!("error: {e:?}"),
    }
}

pub fn alpha(mut e: CmdEntry) {
    match e.into_expr() {
        Ok(expr) => {
            println!("{}", expr.alpha_reduced());
        }
        Err(e) => eprintln!("error: {e:?}"),
    }
}

pub fn closed(mut e: CmdEntry) {
    match e.into_expr() {
        Ok(expr) => {
            // Repl::print_closed(&expr);
            todo!()
        }
        Err(e) => eprintln!("error: {e:?}"),
    }
}

pub fn debrejin(mut e: CmdEntry) {
    match e.into_expr() {
        Ok(l) => {
            println!("{}", l.clone().debrejin_reduced());
            e.repl.runner.mode.bench("printing", || {
                e.repl.print(&l);
            });
        }
        Err(e) => {
            eprintln!("error: {e:?}");
        }
    }
}

pub fn fix_point(e: CmdEntry) {
    e.repl
        .runner
        .mode
        .bench("fix point", || match Scope::from_str(&e.inputs.join(" ")) {
            Ok(s) => {
                Scope::solve_recursion(&s.aliases[0], &s.defs[0]).map_or_else(
                    || println!("{} = {}", s.aliases[0], s.defs[0]),
                    |imp| println!("{} = {imp}", s.aliases[0]),
                );
            }
            Err(e) => eprintln!("error while parsing scope: {e:?}"),
        })
}

pub fn len(mut e: CmdEntry) {
    match e.into_expr() {
        Ok(l) => {
            e.repl.runner.mode.bench("printing", || {
                println!("{}", l.len());
            });
        }
        Err(e) => {
            eprintln!("error: {e:?}");
        }
    }
}

pub fn straight(mut e: CmdEntry) {
    match e.into_expr() {
        Ok(l) => {
            let mut l = l.clone();
            e.repl.runner.mode.bench("beta straight reducing", || {
                l.straight_redex();
            });
            e.repl.runner.mode.bench("printing", || e.repl.print(&l))
        }
        Err(e) => {
            eprintln!("error: {e:?}");
        }
    }
}

use std::str::FromStr;

use super::{CmdEntry, Repl};
use church::{scope::Scope, Body};

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
            Repl::print_closed(&expr);
        }
        Err(e) => eprintln!("error: {e:?}"),
    }
}

pub fn debrejin(mut e: CmdEntry) {
    match e.into_expr() {
        Ok(l) => {
            println!("{}", l.clone().debrejin_reduced());
            e.repl.mode.bench("printing", || {
                e.repl.print_value(&l);
            });
        }
        Err(e) => {
            eprintln!("error: {e:?}");
        }
    }
}

pub fn fix_point(e: CmdEntry) {
    e.repl
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

use super::CmdEntry;
use crate::repl::HashSet;
use church::{straight::StraightRedex, Body, Term};

pub fn printl(mut e: CmdEntry) {
    let lvl = match e.inputs[0].parse::<usize>() {
        Ok(n) => n,
        Err(e) => {
            eprintln!("error: {e:?}");
            return;
        }
    };
    match e.into_expr(1..) {
        Ok(expr) => println!(
            "{}",
            e.repl
                .runner
                .ui
                .format_in_level(&e.repl.runner.scope, &expr, lvl)
        ),
        Err(e) => eprintln!("error: {e:?}"),
    }
}

pub fn delta(mut e: CmdEntry) {
    match e.into_expr(..) {
        Ok(expr) => {
            println!("{}", expr);
        }
        Err(e) => eprintln!("error: {e:?}"),
    }
}

pub fn alpha_eq(mut e: CmdEntry) {
    match e.into_expr(..) {
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
    match e.into_expr(..) {
        Ok(expr) => {
            println!("{}", expr.alpha_reduced());
        }
        Err(e) => eprintln!("error: {e:?}"),
    }
}

pub fn closed(mut e: CmdEntry) {
    fn print_closed(e: &CmdEntry, t: &Term, lvl: usize) {
        println!(
            "{}{}: {}",
            "  ".repeat(lvl),
            e.repl.format_value(t),
            t.closed
        );
        match t.body.as_ref() {
            Body::Id(..) => (),
            Body::App(lhs, rhs) => {
                print_closed(e, lhs, lvl + 1);
                print_closed(e, rhs, lvl + 1);
            }
            Body::Abs(_, l) => print_closed(e, l, lvl + 1),
        }
    }

    match e.into_expr(..) {
        Ok(expr) => {
            print_closed(&e, &expr, 0);
        }
        Err(e) => eprintln!("error: {e:?}"),
    }
}

pub fn debrejin(mut e: CmdEntry) {
    match e.into_expr(..) {
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

pub fn fix_point(_: CmdEntry) {
    // e.repl
    //     .runner
    //     .mode
    //     .bench("fix point", || match Scope::from_str(&e.inputs.join(" ")) {
    //         Ok(s) => {
    //             Scope::solve_recursion(&s.aliases[0], &s.defs[0]).map_or_else(
    //                 || println!("{} = {}", s.aliases[0], s.defs[0]),
    //                 |imp| println!("{} = {imp}", s.aliases[0]),
    //             );
    //         }
    //         Err(e) => eprintln!("error while parsing scope: {e:?}"),
    //     })
    todo!()
}

pub fn len(mut e: CmdEntry) {
    match e.into_expr(..) {
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
    match e.into_expr(..) {
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

pub fn cycle(mut e: CmdEntry) {
    let mut steps = HashSet::default();
    let mut l = match e.into_expr(..) {
        Ok(l) => l,
        Err(e) => {
            eprintln!("error: {e:?}");
            return;
        }
    };
    while l.beta_redex_step() {
        l.debrejin_redex();
        if steps.contains(&l) {
            e.repl.print(&l);
            return;
        } else {
            steps.insert(l.clone());
        }
    }
}

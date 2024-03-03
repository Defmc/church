use std::rc::Rc;

use crate::{Body, Term};

pub trait StraightRedex
where
    Self: Sized,
{
    fn straight_redex(&mut self);
    fn straight_reduced(mut self) -> Self {
        self.straight_redex();
        self
    }
}

impl StraightRedex for Term {
    fn straight_redex(&mut self) {
        match Rc::make_mut(&mut self.body) {
            Body::Id(..) => (),
            Body::App(ref mut f, ref mut x) => {
                if matches!(*f.body, Body::Abs(..)) {
                    f.fix_captures(x);
                    let (id, l) = f.as_mut_abs().unwrap();
                    l.apply_by(*id, x);
                    *self = l.clone();
                    self.straight_redex();
                } else {
                    f.straight_redex();
                    x.straight_redex();
                }
            }
            Body::Abs(..) => {
                self.eta_redex_step();
                self.as_mut_abs().unwrap().1.straight_redex();
            }
        }
    }

    fn straight_reduced(mut self) -> Self {
        self.straight_redex();
        self
    }
}

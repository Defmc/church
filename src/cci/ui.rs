use church::{Body, Term, VarId};

use super::scope::Scope;

#[derive(Debug, Clone)]
pub struct Ui {
    pub readable: bool,
    pub bina_ext: bool,
}

impl Default for Ui {
    fn default() -> Self {
        Self {
            readable: true,
            bina_ext: false,
        }
    }
}

impl Ui {
    pub fn print(&self, s: &Scope, t: &Term) {
        println!("{}", self.format_value(s, t));
    }

    pub fn format_in_level(&self, s: &Scope, t: &Term, lvl: usize) -> String {
        if lvl == 0 {
            return self.format_value(s, t);
        }
        match t.body.as_ref() {
            Body::Id(id) => church::id_to_str(*id),
            Body::App(lhs, rhs) => format!(
                "{} {}",
                self.format_in_level(s, lhs, lvl - 1),
                self.format_in_level(s, rhs, lvl - 1)
            ),
            Body::Abs(v, l) => format!(
                "󰘧{}.({})",
                church::id_to_str(*v),
                self.format_in_level(s, l, lvl - 1)
            ),
        }
    }

    pub fn natural_from_church_encoding(s: &Term) -> Option<usize> {
        fn get_natural(f: VarId, x: VarId, s: &Term) -> Option<usize> {
            if let Body::App(lhs, rhs) = s.body.as_ref() {
                if *lhs.body == Body::Id(f) {
                    return get_natural(f, x, rhs).map(|n| n + 1);
                }
            } else if let Body::Id(v) = s.body.as_ref() {
                return (*v == x).then_some(0);
            }

            None
        }

        if let Body::Abs(f, l) = s.body.as_ref() {
            if let Body::Abs(x, l) = l.body.as_ref() {
                return get_natural(*f, *x, l);
            }
            if *l.body == Body::Id(*f) {
                // λf.(λx.(f x))
                // λf.(f) # eta-reduced version of 1
                return Some(1);
            }
        }
        None
    }

    pub fn beautify(&self, s: &Scope, t: &Term) -> Option<String> {
        if !self.readable {
            return Some(format!("{t}"));
        } else if let Some(alias) = s.get_like(t) {
            return Some(alias.to_string());
        } else if !self.bina_ext {
            if let Some(n) = Self::natural_from_church_encoding(t) {
                return Some(n.to_string());
            }
        } else if let Some(v) = Self::from_list(t) {
            if self.bina_ext {
                if let Some(bin_n) = Self::from_binary_number(&v) {
                    return Some(format!("{bin_n}"));
                }
            }
            let s = format!(
                "[{}]",
                v.into_iter()
                    .map(|e| self.format_value(s, &e))
                    .collect::<Vec<_>>()
                    .join(", ")
            );
            return Some(s);
        };
        None
    }

    pub fn format_value(&self, s: &Scope, t: &Term) -> String {
        if let Some(s) = self.beautify(s, t) {
            return s;
        }
        return match t.body.as_ref() {
            Body::Id(id) => church::id_to_str(*id),
            Body::App(ref f, ref x) => format!("{} {}", self.format_value(s, f), {
                if let Some(s) = self.beautify(s, x) {
                    s
                } else if usize::from(x.len()) > 1 {
                    format!("({})", self.format_value(s, x))
                } else {
                    self.format_value(s, x)
                }
            }),
            Body::Abs(v, l) => {
                format!("λ{}.({})", church::id_to_str(*v), self.format_value(s, l))
            }
        };
    }

    pub fn from_list(b: &Term) -> Option<Vec<Term>> {
        if let Body::Abs(wrapper, b) = b.body.as_ref() {
            if let Body::App(b, rhs) = b.body.as_ref() {
                if let Body::App(wrap, lhs) = b.body.as_ref() {
                    if &Body::Id(*wrapper) == wrap.body.as_ref() {
                        let mut v = vec![lhs.clone()];
                        if let Some(tail) = Self::from_list(rhs) {
                            v.extend(tail);
                        } else {
                            v.push(rhs.clone());
                        }
                        return Some(v);
                    }
                }
            }
        }
        None
    }

    pub fn get_true() -> Term {
        Term::new(Body::Abs(
            0,
            Term::new(Body::Abs(1, Term::new(Body::Id(0)))),
        ))
    }

    pub fn get_false() -> Term {
        Term::new(Body::Abs(
            0,
            Term::new(Body::Abs(1, Term::new(Body::Id(1)))),
        ))
    }

    pub fn from_binary_number(list: &[Term]) -> Option<u128> {
        if list.first()?.alpha_eq(&Self::get_true()) {
            let buf = Self::from_binary_number(&list[1..]).unwrap_or(0) << 1;
            Some(1 + buf)
        } else if list.first()?.alpha_eq(&Self::get_false()) {
            let buf = Self::from_binary_number(&list[1..]).unwrap_or(0) << 1;
            Some(buf)
        } else {
            None
        }
    }
}

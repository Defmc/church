use std::str::FromStr;

use church::{Body, Term, VarId};

use super::scope::Scope;

#[derive(Debug, Clone)]
pub struct Ui {
    readable: bool,
    bina_ext: bool,
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

    pub fn format_value(&self, s: &Scope, t: &Term) -> String {
        if self.readable {
            if let Some(alias) = s.get_like(t) {
                return alias.to_string();
            }
            if !self.bina_ext {
                if let Some(n) = Self::natural_from_church_encoding(t) {
                    return n.to_string();
                }
            }
            if let Some(v) = Self::from_list(t) {
                if self.bina_ext {
                    if let Some(bin_n) = Self::from_binary_number(&v) {
                        return format!("{bin_n}");
                    }
                }
                return format!(
                    "[{}]",
                    v.into_iter()
                        .map(|e| self.format_value(s, &e))
                        .collect::<Vec<_>>()
                        .join(", ")
                );
            }
            return match t.body.as_ref() {
                Body::Id(id) => church::id_to_str(*id),
                Body::App(ref f, ref x) => format!(
                    "{} {}",
                    self.format_value(s, t),
                    if usize::from(x.len()) > 1 {
                        format!("({})", self.format_value(s, t))
                    } else {
                        self.format_value(s, t)
                    }
                ),
                Body::Abs(v, l) => {
                    format!("λ{}.({})", church::id_to_str(*v), self.format_value(s, l))
                }
            };
        }
        format!("{t}")
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

    pub fn from_binary_number(list: &[Term]) -> Option<u128> {
        let one = Term::from_str("^a.(^b.(a))").unwrap();
        let zero = Term::from_str("^a.(^b.(b))").unwrap();
        if list.first()?.alpha_eq(&one) {
            let buf = Self::from_binary_number(&list[1..]).unwrap_or(0) << 1;
            Some(1 + buf)
        } else if list.first()?.alpha_eq(&zero) {
            let buf = Self::from_binary_number(&list[1..]).unwrap_or(0) << 1;
            Some(buf)
        } else {
            None
        }
    }
}

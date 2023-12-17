use crate::{Body, VarId};

pub fn natural(f: VarId, x: VarId, n: usize) -> Body {
    fn natural_body(f: VarId, x: VarId, n: usize) -> Body {
        if n == 0 {
            Body::Id(x)
        } else {
            Body::App(Body::Id(f).into(), natural_body(f, x, n - 1).into())
        }
    }
    natural_body(f, x, n).with([f, x])
}

pub mod bool {
    use crate::Body;

    pub fn t() -> Body {
        Body::Id(0).with([0, 1])
    }

    pub fn f() -> Body {
        Body::Id(1).with([0, 1])
    }

    /// and x y == true, when x == y == true
    pub fn and() -> Body {
        Body::App(
            Body::App(Body::Id(0).into(), Body::Id(1).into()).into(),
            Body::Id(0).into(),
        )
        .with([0, 1])
    }

    /// or x y == false, when x == y == false
    pub fn or() -> Body {
        Body::App(
            Body::App(Body::Id(0).into(), Body::Id(0).into()).into(),
            Body::Id(1).into(),
        )
        .with([0, 1])
    }

    /// inverts the boolean
    /// not true == false
    /// not false == true
    pub fn not() -> Body {
        Body::App(Body::App(Body::Id(0).into(), f().into()).into(), t().into()).with([0])
    }

    /// xor == true, when x != y
    pub fn xor() -> Body {
        let not_otherwise = not().applied([&Body::Id(1)]);
        let and = and().applied([&Body::Id(0), &not_otherwise]);
        or().applied([&and, &Body::Id(1)])
    }

    #[cfg(test)]
    pub mod tests {
        #[test]
        pub fn false_like_zero() {
            let f = super::f();
            let zero = super::super::natural(0, 1, 0);
            assert!(f.alpha_eq(&zero));
        }

        #[test]
        pub fn and_true_false() {
            let mut and = super::and().applied([&super::t(), &super::f()]);
            and.alpha_redex();
            and.beta_redex();
            assert!(and.alpha_eq(&super::f()));
        }

        #[test]
        pub fn and_true_false_no_reduced() {
            let mut and = super::and().applied([&super::t(), &super::f()]);
            and.beta_redex();
            assert!(and.alpha_eq(&super::f()));
        }
    }
}

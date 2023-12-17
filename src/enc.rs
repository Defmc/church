use crate::{Body, VarId};

#[must_use]
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

    #[must_use]
    pub fn t() -> Body {
        Body::Id(0).with([0, 1])
    }

    #[must_use]
    pub fn f() -> Body {
        Body::Id(1).with([0, 1])
    }

    /// and x y == true, when x == y == true
    #[must_use]
    pub fn and() -> Body {
        Body::App(
            Body::App(Body::Id(0).into(), Body::Id(1).into()).into(),
            Body::Id(0).into(),
        )
        .with([0, 1])
    }

    /// or x y == false, when x == y == false
    #[must_use]
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
    #[must_use]
    pub fn not() -> Body {
        Body::App(Body::App(Body::Id(0).into(), f().into()).into(), t().into()).with([0])
    }

    /// xor == true, when x != y
    #[must_use]
    pub fn xor() -> Body {
        let not_otherwise = not().applied([&Body::Id(1)]);
        let and = and().applied([&Body::Id(0), &not_otherwise]);
        or().applied([&and, &Body::Id(1)])
    }

    #[cfg(test)]
    pub mod tests {
        use crate::Body;

        fn bool_to_body(b: bool) -> Body {
            if b {
                super::t()
            } else {
                super::f()
            }
        }

        fn test_case(f: fn() -> Body, l: bool, r: bool) -> Body {
            f().applied([&bool_to_body(l), &bool_to_body(r)])
                .alpha_reduced()
                .beta_reduced()
        }

        const AND_LOGIC_TABLE: &[(bool, bool, bool)] = &[
            /* (a, b, output) */
            (false, false, false),
            (true, false, false),
            (false, true, false),
            (true, true, true),
        ];

        #[test]
        pub fn false_like_zero() {
            let f = super::f();
            let zero = super::super::natural(0, 1, 0);
            assert!(f.alpha_eq(&zero));
        }

        #[test]
        pub fn and() {
            for (l, r, out) in AND_LOGIC_TABLE {
                assert!(test_case(super::and, *l, *r).alpha_eq(&bool_to_body(*out)))
            }
        }

        #[test]
        pub fn and_true_false_no_reduced() {
            let mut and = super::and().applied([&super::t(), &super::f()]);
            and.beta_redex();
            assert!(and.alpha_eq(&super::f()));
        }
    }
}

pub mod naturals {
    use crate::Body;

    #[must_use]
    pub fn natural(n: usize) -> Body {
        fn natural_body(n: usize) -> Body {
            if n == 0 {
                Body::Id(1)
            } else {
                Body::App(Body::Id(0).into(), natural_body(n - 1).into())
            }
        }
        natural_body(n).with([0, 1])
    }

    #[must_use]
    /// succ n f x = n f (f x) = f (n f x)
    pub fn succ() -> Body {
        Body::App(
            Body::Id(1).into(),
            Body::Id(0).in_app(Body::Id(1)).in_app(Body::Id(2)).into(),
        )
        .with([0, 1, 2])
    }

    /// add n m f x = n + m
    // #[must_use]
    // pub fn add() -> Body {}

    #[cfg(test)]
    pub mod tests {
        #[test]
        pub fn false_like_zero() {
            let f = super::super::bool::f();
            let zero = super::natural(0);
            assert!(f.alpha_eq(&zero));
        }
    }
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
        Body::Id(0)
            .in_app(Body::Id(1))
            .in_app(Body::Id(0))
            .with([0, 1])
    }

    /// or x y == false, when x == y == false
    #[must_use]
    pub fn or() -> Body {
        Body::Id(0)
            .in_app(Body::Id(0))
            .in_app(Body::Id(1))
            .with([0, 1])
    }

    /// inverts the boolean
    /// not true == false
    /// not false == true
    #[must_use]
    pub fn not() -> Body {
        Body::Id(0).in_app(f()).in_app(t()).with([0])
    }

    /// xor == true, when x != y
    #[must_use]
    pub fn xor() -> Body {
        let not_otherwise = not().applied([&Body::Id(1)]);
        Body::Id(0)
            .in_app(not_otherwise)
            .in_app(Body::Id(1))
            .with([0, 1])
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
                .beta_reduced()
        }

        const AND_LOGIC_TABLE: &[(bool, bool, bool)] = &[
            /* (a, b, output) */
            (false, false, false),
            (true, false, false),
            (false, true, false),
            (true, true, true),
        ];

        const OR_LOGIC_TABLE: &[(bool, bool, bool)] = &[
            /* (a, b, output) */
            (false, false, false),
            (true, false, true),
            (false, true, true),
            (true, true, true),
        ];

        const XOR_LOGIC_TABLE: &[(bool, bool, bool)] = &[
            /* (a, b, output) */
            (false, false, false),
            (true, false, true),
            (false, true, true),
            (true, true, false),
        ];

        const NOT_LOGIC_TABLE: &[(bool, bool)] =
            &[/* (a, output) */ (false, true), (true, false)];

        #[test]
        pub fn and() {
            for (l, r, out) in AND_LOGIC_TABLE {
                assert!(
                    test_case(super::and, *l, *r).alpha_eq(&bool_to_body(*out)),
                    "{} != {}",
                    test_case(super::and, *l, *r),
                    bool_to_body(*out)
                )
            }
        }
        #[test]
        pub fn nand() {
            fn nand() -> Body {
                let and = super::and().in_app(Body::Id(0)).in_app(Body::Id(1));
                let not = super::not().in_app(and);
                not.with([0, 1]).beta_reduced()
            }
            for (l, r, out) in AND_LOGIC_TABLE {
                assert!(test_case(nand, *l, *r).alpha_eq(&bool_to_body(!*out)))
            }
        }

        #[test]
        pub fn and_transitivity() {
            for (l, r, out) in AND_LOGIC_TABLE {
                assert!(test_case(super::and, *r, *l).alpha_eq(&bool_to_body(*out)))
            }
        }

        #[test]
        pub fn or() {
            for (l, r, out) in OR_LOGIC_TABLE {
                assert!(test_case(super::or, *l, *r).alpha_eq(&bool_to_body(*out)))
            }
        }

        #[test]
        pub fn nor() {
            fn nor() -> Body {
                let or = super::or().in_app(Body::Id(0)).in_app(Body::Id(1));
                let not = super::not().in_app(or);
                not.with([0, 1]).beta_reduced()
            }
            for (l, r, out) in OR_LOGIC_TABLE {
                assert!(test_case(nor, *l, *r).alpha_eq(&bool_to_body(!*out)))
            }
        }

        #[test]
        pub fn or_transitivity() {
            for (l, r, out) in OR_LOGIC_TABLE {
                assert!(test_case(super::or, *r, *l).alpha_eq(&bool_to_body(*out)))
            }
        }

        #[test]
        pub fn xor() {
            for (l, r, out) in XOR_LOGIC_TABLE {
                assert!(test_case(super::xor, *l, *r).alpha_eq(&bool_to_body(*out)))
            }
        }

        #[test]
        pub fn xor_transitivity() {
            for (l, r, out) in XOR_LOGIC_TABLE {
                assert!(test_case(super::xor, *r, *l).alpha_eq(&bool_to_body(*out)))
            }
        }

        #[test]
        pub fn not() {
            for (l, out) in NOT_LOGIC_TABLE {
                assert!(super::not()
                    .applied([&bool_to_body(*l)])
                    .beta_reduced()
                    .alpha_eq(&bool_to_body(*out)))
            }
        }

        #[test]
        pub fn xor_as_or_and_not_and() {
            fn new_xor() -> Body {
                let and = super::and().in_app(Body::Id(0)).in_app(Body::Id(1));
                let nand = super::not().in_app(and);
                let or = super::or().in_app(Body::Id(0)).in_app(Body::Id(1));
                let xor = super::and().in_app(or).in_app(nand);
                xor.with([0, 1]).beta_reduced()
            }
            for (l, r, out) in XOR_LOGIC_TABLE {
                assert!(
                    test_case(new_xor, *l, *r).alpha_eq(&bool_to_body(*out)),
                    "{}",
                    test_case(new_xor, *l, *r)
                )
            }
        }
    }
}

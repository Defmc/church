use crate::{Body, VarId};

pub fn natural(f: VarId, x: VarId, n: usize) -> Body {
    fn natural_body(f: VarId, x: VarId, n: usize) -> Body {
        if n == 0 {
            Body::Id(x)
        } else {
            Body::App(Body::Id(f).into(), natural_body(f, x, n - 1).into())
        }
    }
    natural_body(f, x, n).with([f, x].into_iter().peekable())
}

pub mod bool {
    use crate::{Body, VarId};

    pub fn t() -> Body {
        Body::Id(0).with([0, 1].into_iter().peekable())
    }

    pub fn f() -> Body {
        Body::Id(1).with([0, 1].into_iter().peekable())
    }

    pub fn and() -> Body {
        Body::App(
            Body::App(Body::Id(0).into(), Body::Id(1).into()).into(),
            Body::Id(0).into(),
        )
        .with([0, 1].into_iter().peekable())
    }

    pub fn or() -> Body {
        Body::App(
            Body::App(Body::Id(0).into(), Body::Id(0).into()).into(),
            Body::Id(1).into(),
        )
        .with([0, 1].into_iter().peekable())
    }

    /// inverts the boolean
    /// not(true) == false
    /// not(false) == true
    pub fn not() -> Body {
        Body::App(Body::App(Body::Id(0).into(), f().into()).into(), t().into())
            .with([0].into_iter().peekable())
    }

    pub fn xor() -> Body {
        let not_otherwise = not().applied([&Body::Id(1)].into_iter().peekable());
        let and = and().applied([&Body::Id(0), &not_otherwise].into_iter().peekable());
        or().applied([&and, &Body::Id(1)].into_iter().peekable())
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
            let mut and = super::and().applied([&super::t(), &super::f()].into_iter().peekable());
            println!("and: {and}");
            and.alpha_redex();
            println!("reduced and: {and}");
            and.beta_redex();
            println!("reduced and: {and}");
        }
    }
}

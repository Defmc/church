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

    pub fn t(choice: VarId, otherwise: VarId) -> Body {
        Body::Id(choice).with([choice, otherwise].into_iter().peekable())
    }

    pub fn f(choice: VarId, otherwise: VarId) -> Body {
        Body::Id(otherwise).with([choice, otherwise].into_iter().peekable())
    }

    pub fn and(choice: VarId, otherwise: VarId) -> Body {
        Body::App(
            Body::App(Body::Id(choice).into(), Body::Id(otherwise).into()).into(),
            Body::Id(choice).into(),
        )
        .with([choice, otherwise].into_iter().peekable())
    }

    pub fn or(choice: VarId, otherwise: VarId) -> Body {
        Body::App(
            Body::App(Body::Id(choice).into(), Body::Id(choice).into()).into(),
            Body::Id(otherwise).into(),
        )
        .with([choice, otherwise].into_iter().peekable())
    }

    pub fn not(b: VarId) -> Body {
        Body::App(
            Body::App(Body::Id(b).into(), f(0, 1).into()).into(),
            t(0, 1).into(),
        )
    }
}

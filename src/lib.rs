pub mod term;

pub use term::{Body, Term};

#[macro_export]
macro_rules! assert_alpha_eq {
    ($left:expr, $right:expr $(,)?) => {
        $crate::assert_alpha_eq!($left, $right, "{} is alpha-different from {}", $left, $right)
     };
    ($left:expr, $right:expr, $($arg:tt)+) => {{
        assert_eq!($left.coerce(Term::unique_alpha_redex), $right.coerce(Term::unique_alpha_redex), $($arg)+);
     }};
}

#[macro_export]
macro_rules! assert_alpha_ne {
    ($left:expr, $right:expr $(,)?) => {
        assert_ne!($left.coerce(Term::unique_alpha_redex),
                   $right.coerce(Term::unique_alpha_redex))
     };
    ($left:expr, $right:expr, $($arg:tt)+) => {
        assert_ne!($left.coerce(Term::unique_alpha_redex),
                   $right.coerce(Term::unique_alpha_redex), $($arg)+)
     };
}

#[cfg(test)]
mod tests {
    use crate::{Body, Term};

    #[test]
    fn id_formatting() {
        let id: Term = Body::Abs(0, Body::Var(0).into()).into();
        if cfg!(feature = "aliased-vars") {
            assert_eq!(id.to_string(), "λα.α");
        } else {
            assert_eq!(id.to_string(), "λ0.0");
        }
    }

    #[test]
    fn uniq_redex() {
        let expr = Term::from(Body::Abs(0, Body::Var(0).into())).coerce(Term::unique_alpha_redex);
        let reduced_expr = Body::Abs(0, Body::Var(0).into()).into();
        assert_eq!(expr, reduced_expr);
    }
}

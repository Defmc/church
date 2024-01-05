use super::{Ast, Gramem, Meta, Sym};
use lrp::Grammar;
use lrp::ReductMap;
#[allow(unused_imports, clippy::enum_glob_use)]
#[must_use]
pub fn grammar() -> Grammar<Sym> {
    Grammar::new(
        Sym::EntryPoint,
        {
            use super::Sym::*;
            use crate::Body;
            let mut map = lrp::RuleMap::new();
            map.insert(
                EntryPoint,
                lrp::grammar::Rule::new(EntryPoint, vec![vec![App]]),
            );
            map.insert(
                Lambda,
                lrp::grammar::Rule::new(
                    Lambda,
                    vec![vec![
                        Sym::LambdaChar,
                        Sym::Var,
                        Sym::Body,
                        Sym::OpenParen,
                        App,
                        Sym::CloseParen,
                    ]],
                ),
            );
            map.insert(
                App,
                lrp::grammar::Rule::new(App, vec![vec![App, Expr], vec![Expr]]),
            );
            map.insert(
                Expr,
                lrp::grammar::Rule::new(
                    Expr,
                    vec![
                        vec![Sym::OpenParen, App, Sym::CloseParen],
                        vec![Lambda],
                        vec![Sym::Var],
                    ],
                ),
            );

            map
        },
        Sym::Eof,
    )
}

#[allow(
    non_snake_case,
    clippy::enum_glob_use,
    unused_braces,
    unused_imports,
    unused_assignments,
    clippy::unnecessary_literal_unwrap
)]
pub fn reduct_map() -> ReductMap<Meta<Ast>, Sym> {
    use super::Sym::*;
    use crate::Body;
    let mut map = lrp::ReductMap::new();

    fn lrp_wop_EntryPoint_0(toks: &[Gramem]) -> lrp::Meta<Ast> {
        lrp::Meta::new(
            {
                let e = toks[0].clone();
                {
                    Ast::Expr(e.item.item.as_expr().clone())
                }
            },
            lrp::Span::new(toks[0].item.span.start, toks.last().unwrap().item.span.end),
        )
    }
    map.insert(EntryPoint, vec![lrp_wop_EntryPoint_0]);

    fn lrp_wop_Lambda_0(toks: &[Gramem]) -> lrp::Meta<Ast> {
        lrp::Meta::new(
            {
                let i = toks[1].clone();
                let a = toks[4].clone();
                {
                    Ast::Expr(Body::Abs(
                        i.item.item.as_var(),
                        a.item.item.as_expr().clone().into(),
                    ))
                }
            },
            lrp::Span::new(toks[0].item.span.start, toks.last().unwrap().item.span.end),
        )
    }
    map.insert(Lambda, vec![lrp_wop_Lambda_0]);

    fn lrp_wop_App_0(toks: &[Gramem]) -> lrp::Meta<Ast> {
        lrp::Meta::new(
            {
                let a = toks[0].clone();
                let e = toks[1].clone();
                {
                    Ast::Expr(Body::App(
                        a.item.item.as_expr().clone().into(),
                        e.item.item.as_expr().clone().into(),
                    ))
                }
            },
            lrp::Span::new(toks[0].item.span.start, toks.last().unwrap().item.span.end),
        )
    }
    fn lrp_wop_App_1(toks: &[Gramem]) -> lrp::Meta<Ast> {
        lrp::Meta::new(
            {
                let e = toks[0].clone();
                {
                    Ast::Expr(e.item.item.as_expr().clone())
                }
            },
            lrp::Span::new(toks[0].item.span.start, toks.last().unwrap().item.span.end),
        )
    }
    map.insert(App, vec![lrp_wop_App_0, lrp_wop_App_1]);

    fn lrp_wop_Expr_0(toks: &[Gramem]) -> lrp::Meta<Ast> {
        lrp::Meta::new(
            {
                let e = toks[1].clone();
                {
                    Ast::Expr(e.item.item.as_expr().clone())
                }
            },
            lrp::Span::new(toks[0].item.span.start, toks.last().unwrap().item.span.end),
        )
    }
    fn lrp_wop_Expr_1(toks: &[Gramem]) -> lrp::Meta<Ast> {
        lrp::Meta::new(
            {
                let l = toks[0].clone();
                {
                    Ast::Expr(l.item.item.as_expr().clone())
                }
            },
            lrp::Span::new(toks[0].item.span.start, toks.last().unwrap().item.span.end),
        )
    }
    fn lrp_wop_Expr_2(toks: &[Gramem]) -> lrp::Meta<Ast> {
        lrp::Meta::new(
            {
                let i = toks[0].clone();
                {
                    Ast::Expr(Body::Id(i.item.item.as_var()))
                }
            },
            lrp::Span::new(toks[0].item.span.start, toks.last().unwrap().item.span.end),
        )
    }
    map.insert(Expr, vec![lrp_wop_Expr_0, lrp_wop_Expr_1, lrp_wop_Expr_2]);

    map
}

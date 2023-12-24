## α reducing free variables (fixed)
```
// `a` shouldn't be used, as there is another free alias to it
\> ^b.(a a b)             
        expr:    λb.(a a b)
        α-redex: λa.(a a a)
                -> β:  λa.(a a a)
        β-redex: λb.(a a b)
                -> α:  λa.(a a a)

// should return `a`
\> ^b.(a) c
        expr:    λb.(a) c
        α-redex: λa.(a) c
                -> β:  c
        β-redex: a
                -> α:  a
```
Possible approaches:
- [x] Add a "free variable" field in map while alpha reducing. If it's a free variable, but there's already another alias to a non-free, the reductor should re-map the non-free to another letter. In this way, alpha reducing will be a O(2) algorithm, instead of a current O(1) implementation.

```
\> ^a.(a) ^a.(b) 
        expr:    λa.(a) (λa.(b))
        α-redex: λb.(b) (λc.(b))
                -> β:  λc.(b)
        β-redex: λa.(b)
                -> α:  λb.(b)
```

## can't parse applications in parenthesis (fixed)
```
\> (λa.(λb.(a a b)) a b)
        error:   UnexpectedToken(Var, [CloseParen])
```
Possible approaches:
- [x] Switch `Expr -> "(" Expr ")"` to `Expr -> "(" App ")"`. The grammar still will be a SLR(1) one.


## The XOR problem
Maybe it's storing the state between the redexes of applications

## Shadowing impossibilitating alpha equivalences (fixed)
```
\> ^a.(^a.(a))
        expr:    λa.(λa.(a))
        α-eq:    false
        α-redex: λa.(λb.(b))
                -> β:  λa.(λb.(b))
        β-redex: λa.(λa.(a))
                -> α:  λa.(λb.(b))

\> ^a.(^b.(b))
        expr:    λa.(λb.(b))
        α-eq:    false // should returned `true`
        α-redex: λa.(λb.(b))
                -> β:  λa.(λb.(b))
        β-redex: λa.(λb.(b))
                -> α:  λa.(λb.(b))
```

- [x] Count binds with a variable, instead of `map.len`, as shadowing is ignored.

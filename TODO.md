### Impl matches
```
λ> ^x.(x)
expr: ^x.(x)
        δ-eq:    true
        δ-redex: ^x.(x)
        α-eq:    false
        α-redex: λx.(x)
                -> β:  λx.(x)
        β-redex: λx.(x)
                -> α:  λx.(x)
        matches: [I]
```

### Impl lambda lifting
```
λ> :lift ^x.(x y)
λy.(λx.(x y))
```

### Impl binary arithmetic

### Beta-reduce scope expressions

### Allows IO with a Sys abstraction

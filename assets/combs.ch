Y = λf.(\x. x x) (\x. f (x x))
Θ = (λx.λy.y (x x y)) (λx.λy.y (x x y))
Θr = (λd.d d) (λx.λy.y (x x y))

I = λx.x
K = λx.λy.x
S = λx.λy.λz.xz (yz)

# Avoids repitition
Let = λa.λf.f a

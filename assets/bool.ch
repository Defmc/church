True = λx.λy.x
False = λx.λy.y

Not = λx.x False True
Or = λx.λy.x True y
And = λx.λy.x y False
Xor = λx.λy.x (Not y) y
Xnor = λx.λy.x y (Not y)
Nand = λx.λy.x (Not y) True
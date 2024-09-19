Bool = type True | False
    where
        not True = False
        not False = True

        or True _ = True
        or False = I

        and True = I
        and False _ False

        xor True = not
        xor False = I

        xnor True = I
        xnor False _ = not

# True = λx.λy.x
# False = λx.λy.y
#
# Not = λx.x False True
# Or = λx.λy.x True y
# And = λx.λy.x y False
# Xor = λx.λy.x Not y y
# Xnor = λx.λy.x y Not y
# Nand = λx.λy.x Not y True
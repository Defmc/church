Pair = λx.λy.λz.z x y
Fst = λl.l λx.λy.x
Snd = λl.l λx.λy.x
        
Cons = Pair
Head = Fst
Tail = Snd
Nil = λ_.λx.λy.x
IsNil = λl.l (λh.λt.λx.λy.y)

Map f l =
    IsNil l
        Nil
        (Cons (f (Head l)) (Map f (Tail l)))

Zip m n = 
    Or (IsNil m) (IsNil n)
        Nil
        (Cons (Pair (Head m) (Head n)) (Zip (Tail m) (Tail n)))

Filter f l = 
    IsNil l
        Nil
        (f (Head l)
            (Cons (Head l) (Filter f (Tail l)))
            (Filter f (Tail l))
        )

Foldr d f l =
    IsNil l
        d
        (f (Pair (Foldr d f (Tail l)) (Head l)))

Foldl d f l =
    IsNil l
        d

    
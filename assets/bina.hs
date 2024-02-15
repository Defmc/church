badd :: [Bool] -> [Bool] -> Bool -> [Bool]
badd x y c =
  if null x && null y
    then if c then [True] else []
    else
      if null x
        then badd y [c] False
        else
          if null y
            then badd x [c] False
            else (((head x) /= (head y)) /= c) : (badd (tail x) (tail y) ((head x) && (head y)))

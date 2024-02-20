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

-- Left shifting
-- bshiftl 1101100 2 = 01100
bshiftl :: [Bool] -> Natural -> [Bool]
bshiftl xs 0 = xs
bshiftl (x:xs) n = bshiftl xs (n - 1)

-- Right shifting
-- bshiftr 1101100 2 = 11011
bshiftr :: [Bool] -> Natural -> [Bool]
bshiftr xs 0 = xs
bshiftr xs n = False : bshiftr xs (n - 1)
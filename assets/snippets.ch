main : IO ()
main = do
    std.io.println "Fact of 3 is {fact 3}"

fact : Nat -> Nat
fact 0 = 1
fact n = n * fact (n - 1)

Person = type where
    name : String = "default value",
    age : Nat,
    sons : List Person

author = Person { name = "defmc", age = 18, sons: [] }

Result : α -> β -> α ⊕ β
Result : Type -> Type -> Type
Result = type Ok o | Err e
    where
        unwrap Ok v = v
        unwrap Err _ = panic "unwrapping an `Err` variant"

Result.map (Ok v : Self) (f : α -> β) (_ : β) : β = Ok f v
Result.map self ... = self

Result.map_or (self : Self) (f : α -> β) (d : β) : β = match self with
    | Ok v => f v
    | Err _ => d

Option = type Some v | None
# function's type declaration
id : α -> α

# with implicit argument
id { α : Type } (x : α) : α = x

# with type
id2 (x : α) : α = x
id3 x = x
id4 = I

long_fn : Nat -> Nat -> Nat -> Nat -> Nat -> Nat
long_fn 1 ... = 1
long_fn ... 1 = 1
long_fn 1 ... 1 = 2
long_fn 1 1 ... 1 = 3
long_fn 1 1 ... 1 1 = 4
long_fn ... = 0

just_when_eq (l : Nat) (r : Nat) : Nat = 0
just_when_eq n n = n

# value unwrapping
map (Some x) f = Some $ f x
map None f = None

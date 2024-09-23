# a simple sum type
Option = type Some α | None
Option = type Some α | None
    where # to define properties
        unwrap Some x = x
        unwrap None = sys.proc.panic "unwrapping a `None` value"

# explicit typed sum type declaration
# FIXME: Do I really needs to support this type definition?
Option : Type -> Type # it's a type constructor: needs a type to build one

# or...
Option : α -> α ⊕ () # builds a sum type with two variants: Unit or α 

# Product type constructor with one generic  
Person = type α where
    name : String # FIXME: `Str` or `String`?
    age : Nat
    gift : α
    # methods are "just" properties with pre-defined values and compile-time resolution. I.e are not part of the type.
    greetings Self { name, age } = "Hi, I'm {name} whose age's {age}."

if exists("b:current_syntax")
    finish
endif

syn match qkType '[A-ZA-Z0-9Α-Ω_][a-zA-Z0-9α-ωΑ-Ω_]*'
hi def link qkType Type

syn match qkIdent '[α-κμ-ωa-z_][a-zA-Z0-9α-ωΑ-Ω_]\+'
hi def link qkIdent Identifier

syn match qkSpecial '\\\d\d\d\|\\.'
hi def link qkSpecial Special

syn match qkInt '[-+]\d\+'
syn match qkInt '\d\+'
hi def link qkInt Number 

syn match qkFloat '\d\+\.\d*'
syn match qkFloat '[-+]\d\+\.\d*'
hi def link qkFloat Float 

syn region qkString start=+f"+  skip=+\\\\\|\\"+  end=+"+
syn region qkString start=+"+  skip=+\\\\\|\\"+  end=+"+
hi def link qkString String

hi def link qkBoolean Boolean
syn keyword qkBoolean true false

syn keyword qkStructure type
hi def link qkStructure Structure

syn keyword qkConditional if then elif else match
hi def link qkConditional Conditional

syn keyword qkLabel where let in with
hi def link qkLabel Label

syn keyword qkOperator fn
hi def link qkOperator Operator

syn keyword qkFunc use
hi def link qkFunc Function
 
syn keyword qkTodo TODO FIXME NOTE SAFE PROOF contained
hi def link qkTodo Todo

syn match qkComment "#[^#\n]*[#\n]" contains=qkTodo
hi def link qkComment Comment

syn match qkMacro "#[a-zA-Z]\+"
hi def link qkMacro Macro

let b:current_syntax="church"

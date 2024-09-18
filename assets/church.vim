if exists("b:current_syntax")
    finish
endif

syn match qkType '[A-ZA-Z0-9Α-Ω_][a-zA-Z0-9α-ωΑ-Ω_]*'
syn match qkIdent '[α-κμ-ωa-z_][a-zA-Z0-9α-ωΑ-Ω_]\+'
syn match qkSpecial '\\\d\d\d\|\\.'
syn match qkInt '[-+]\d\+'
syn match qkInt '\d\+'
syn region qkString start=+f"+  skip=+\\\\\|\\"+  end=+"+
syn region qkString start=+"+  skip=+\\\\\|\\"+  end=+"+

syn match qkFloat '\d\+\.\d*'
syn match qkFloat '[-+]\d\+\.\d*'

syn keyword qkStructure type
syn keyword qkConditional if then elif else match
syn keyword qkLabel where let in 
syn keyword qkOperator use fn type
syn keyword qkBoolean true false
 
syn keyword qkTodo TODO FIXME NOTE SAFE PROOF contained
syn match qkComment "#[^#\n]*[#\n]" contains=qkTodo
syn match qkMacro "#[a-zA-Z]\+"

hi def link qkInt Number 
hi def link qkFloat Float 
hi def link qkBoolean Boolean
hi def link qkString String
hi def link qkIdent Identifier

hi def link qkLabel Label
hi def link qkOperator Operator
hi def link qkConditional Conditional

hi def link qkTodo Todo
hi def link qkMacro Macro
hi def link qkSpecial Special
hi def link qkComment Comment

let b:current_syntax="church"

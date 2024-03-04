" if exists("b:current_syntax")
"     finish
" endif

syn match churchOperator "[=^(->)\(\)\.\\]"
syn match churchVar "[a-z]"
syn match churchAlias "\w\w\+"
syn match churchComment "#.*$"
syn match churchKeyword ":\w\+"

let b:current_syntax = "church"

hi def link churchKeyword Keyword
hi def link churchVar Identifier
hi def link churchAlias Constant
hi def link churchOperator Operator
hi def link churchComment Comment

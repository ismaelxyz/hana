" Vim syntax file
" Language:         Hana
" Maintainer:       asraelxyz <ismaelb02@hotmail.com>
" Latest Revision:  2021-06-01
" Changes:          2021-06-01 Initial version
"syn match   keyword2 '\c.*\<\(true\|false\|nil\|self\).*'
"hi def link keyword    Constant

if exists("b:current_syntax")
  finish
endif
syntax keyword Normal rand or not use begin project end if else while for continue break elsif try case as raise in match func fn then return record

syntax region HString 	start=+\z(["']\)+  skip=+\\\%(\z1\|$\)+  end=+\z1+
syntax match  number 	/\c\<\%(\d\+\%(e[+-]\=\d\+\)\=\|0b[01]\+\|0o\o\+\|0x\%(\x\|_\)\+\)n\=\>/

" Comments
syntax region  HComment        start=+//+ end=/$/
syntax region  HComment        start=+/\*+  end=+\*/+

hi def link HString    String
hi def link HComment   Comment
hi def link number 	  Number
hi def link Normal 	  Keyword


let b:current_syntax = "hana"

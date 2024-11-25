# kaleidoscope

From [llvm](https://llvm.org/docs/index.html) tutorial - [MyFirstLanguageFrontend](https://llvm.org/docs/tutorial/MyFirstLanguageFrontend/index.html)

Some code from [inkwell/example/kaleidoscope](https://github.com/TheDan64/inkwell/blob/master/examples/kaleidoscope/implementation_typed_pointers.rs)

## About [`llvm-sys`](https://crates.io/crates/llvm-sys)

If you get error from `llvm-sys` when compile this project, you can check my blog about [how to init llvm-sys](https://studylessshape.github.io/post/rust/how-to-init-llvm-sys/)

## bnf collect
```bnf
definition ::= 'def' prototype expression
external ::= 'extern' prototype
prototype
    ::= id '(' id* ')'

expression ::= primary binoprhs
binoprhs
    ::= ('+' primary)*
primary
    ::= identifierexpr
    ::= numberexpr
    ::= parentexpr
numberexpr ::= number
parentexpr
    ::= '(' expression ')'
identifierexpr
    ::= identifier
    ::= identifier '(' expression* ')'
```
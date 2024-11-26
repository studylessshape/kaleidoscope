use std::{
    cell::RefCell,
    io::{stdin, stdout, Result, Write},
};

use kaleidoscope_rs::{ast::Codegen, compile::Compiler, lex::Token, parser::Parser};

fn main() -> Result<()> {
    let mut stdout = stdout().lock();
    let compiler = Compiler::default();

    let rc_compiler = RefCell::new(compiler);
    loop {
        write!(&mut stdout, "> ")?;
        stdout.flush()?;

        let mut stmt = String::new();
        stdin().read_line(&mut stmt)?;

        let parser = Parser::new(stmt).map_err(std::io::Error::other)?;
        let res = match parser.peek() {
            Token::Def => handle_definition(parser, &mut rc_compiler.borrow_mut()),
            Token::Extern => handle_extern(parser, &mut rc_compiler.borrow_mut()),
            Token::EOF => {
                println!("{}", rc_compiler.borrow().print_module());
                continue;
            }
            _ => handle_toplevel(parser, &mut rc_compiler.borrow_mut()),
        };

        if let Err(e) = res {
            println!("Error: {}", e);
        }
    }
}

fn handle_definition<S>(mut parser: Parser<S>, compiler: &mut Compiler) -> Result<()>
where
    S: AsRef<[u8]>,
{
    println!("Parse a function define");

    let ast = parser.parse_definition().map_err(std::io::Error::other)?;
    // println!("{:?}", ast);

    match ast.codegen(compiler) {
        Ok(val) => println!("{}", compiler.print(val)),
        Err(err) => println!("err: {:?}", err),
    }
    Ok(())
}

fn handle_extern<S>(mut parser: Parser<S>, compiler: &mut Compiler) -> Result<()>
where
    S: AsRef<[u8]>,
{
    println!("Parse an extern");

    let ast = parser.parse_extern().map_err(std::io::Error::other)?;
    // println!("{:?}", ast);

    match ast.codegen(compiler) {
        Ok(val) => println!("{}", compiler.print(val)),
        Err(err) => println!("err: {:?}", err),
    }
    Ok(())
}

fn handle_toplevel<S>(mut parser: Parser<S>, compiler: &mut Compiler) -> Result<()>
where
    S: AsRef<[u8]>,
{
    println!("Parse top-level expr");

    let ast = parser.parse_toplevel().map_err(std::io::Error::other)?;
    // println!("{:?}", ast);

    match ast.codegen(compiler) {
        Ok(val) => println!("{}", compiler.print(val)),
        Err(err) => println!("err: {:?}", err),
    }
    Ok(())
}

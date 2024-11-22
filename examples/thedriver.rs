use std::io::{stdin, stdout, Result, Write};

use kaleidoscope_rs::{lex::Token, parser::Parser};

fn main() -> Result<()> {
    let mut stdout = stdout().lock();
    
    loop {
        write!(&mut stdout, "> ")?;
        stdout.flush()?;

        let mut stmt = String::new();
        stdin().read_line(&mut stmt)?;

        let parser = Parser::new(stmt).map_err(std::io::Error::other)?;
        let res = match parser.peek() {
            Token::Def => handle_definition(parser),
            Token::Extern => handle_extern(parser),
            Token::EOF => continue,
            _ => handle_toplevel(parser),
        };

        if let Err(e) = res {
            println!("Error: {}", e);
        }
    }
}

fn handle_definition<S>(mut parser: Parser<S>) -> Result<()>
where S: AsRef<[u8]>
{
    println!("Parse a function define");
    println!("{:?}", parser.parse_definition().map_err(std::io::Error::other)?);
    Ok(())
}

fn handle_extern<S>(mut parser: Parser<S>) -> Result<()>
where S: AsRef<[u8]>
{
    println!("Parse an extern");
    println!("{:?}", parser.parse_extern().map_err(std::io::Error::other)?);
    Ok(())
}

fn handle_toplevel<S>(mut parser: Parser<S>) -> Result<()>
where S: AsRef<[u8]>
{
    println!("Parse top-level expr");
    println!("{:?}", parser.parse_toplevel().map_err(std::io::Error::other)?);
    Ok(())
}
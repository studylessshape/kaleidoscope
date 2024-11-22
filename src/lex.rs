use std::{
    io::{Cursor, Read, Seek},
    result,
};

use crate::error::LexError;

type LexResult<T> = result::Result<T, LexError>;

pub enum Token {
    /// keyword `def`
    Def,
    /// keyword `extern`
    Extern,
    /// char `(`
    LeftBracket,
    /// char `)`
    RightBracket,
    /// char `[`
    LeftSquare,
    /// char `]`
    RightSquare,
    /// char `<`
    LeftAngle,
    /// char `>`
    RightAngle,
    /// char `{`
    LeftCurly,
    /// char `}`
    RightCurly,
    /// char `=`
    Assign,
    /// char `+`
    Add,
    /// char `-`
    Minus,
    /// char `*`
    Mul,
    /// char `/`
    Div,
    /// symbol `==`
    Equal,
    /// char '!'
    Exclamation,
    /// symbol `!=`
    NotEq,
    /// symbol `<=`
    LessEq,
    /// symbol `>=`
    GreaEq,
    /// function or variable identifier
    Identifier(String),
    Number(f64),
    String(String),
    EOF,
}

pub struct Lexer<S>
where
    S: AsRef<[u8]>,
{
    input: Cursor<S>,
    ahead: Option<Token>,
}

impl<S> Lexer<S>
where
    S: AsRef<[u8]>,
{
    pub fn new(input: S) -> Self {
        Self {
            input: Cursor::new(input),
            ahead: None,
        }
    }

    pub fn next(&mut self) -> LexResult<Token> {
        match self.ahead.take() {
            Some(t) => Ok(t),
            None => self.do_next(),
        }
    }

    fn do_next(&mut self) -> LexResult<Token> {
        self.skip_whitespace()?;

        if let Some(ch) = self.next_char()? {
            match ch {
                '\'' | '"' => self.read_string(ch),
                '+' => Ok(Token::Add),
                '-' => Ok(Token::Minus),
                '*' => Ok(Token::Mul),
                '/' => Ok(Token::Div),
                '(' => Ok(Token::LeftBracket),
                ')' => Ok(Token::RightBracket),
                '[' => Ok(Token::LeftSquare),
                ']' => Ok(Token::RightSquare),
                '{' => Ok(Token::LeftCurly),
                '}' => Ok(Token::RightCurly),
                '=' => self.read_ahead('=', Token::Equal, Token::Assign),
                '>' => self.read_aheadf('=', |_| Ok(Token::GreaEq), |_| Ok(Token::RightAngle)),
                '<' => self.read_aheadf('=', |_| Ok(Token::LessEq), |_| Ok(Token::LeftAngle)),
                '#' => {
                    self.skip_comment()?;
                    self.do_next()
                },
                '.' | '0'..='9' => self.read_number(ch),
                _ => self.read_identifier(ch),
            }
        } else {
            Ok(Token::EOF)
        }
    }

    fn read_identifier(&mut self, first: char) -> LexResult<Token> {
        let mut identifier = first.to_string();
        while let Some(ch) = self.next_char()? {
            match ch {
                ch if ch.is_alphanumeric() || ch == '_' => identifier.push(ch),
                _ => {
                    self.back_seek()?;
                    break;
                }
            }
        }

        match identifier.as_str() {
            "def" => Ok(Token::Def),
            "extern" => Ok(Token::Extern),
            _ => Ok(Token::Identifier(identifier)),
        }
    }

    fn read_string(&mut self, quote: char) -> LexResult<Token> {
        // let mut quote_count = 1;
        let mut str = String::new();
        while let Some(ch) = self.next_char()? {
            match ch {
                ch if ch == quote => break,
                _ => str.push(ch),
            }
        }

        Ok(Token::String(str))
    }

    fn read_ahead(&mut self, ahead: char, long: Token, short: Token) -> LexResult<Token> {
        match self.next_char()? {
            Some(ch) if ch == ahead => Ok(long),
            None | Some(_)=> {
                self.back_seek()?;
                Ok(short)
            },
        }
    }

    fn read_aheadf<L, Sh>(&mut self, ahead: char, long: L, short: Sh) -> LexResult<Token>
    where
        L: Fn(&mut Self) -> LexResult<Token>,
        Sh: Fn(&mut Self) -> LexResult<Token>,
    {
        match self.next_char()? {
            Some(ch) if ch == ahead => long(self),
            None | Some(_) => {
                self.back_seek()?;
                short(self)
            }
        }
    }

    fn read_number(&mut self, ahead: char) -> LexResult<Token> {
        match ahead {
            _ => {
                let mut num_str = ahead.to_string();
                while let Some(ch) = self.next_char()? {
                    match ch {
                        '0'..='9' | '.' | '_' => num_str.push(ch),
                        _ => {
                            self.back_seek()?;
                            break;
                        }
                    }
                }

                num_str.parse::<f64>().map(|n| Token::Number(n)).map_err(|e| e.into())
            }
        }
    }

    fn skip_comment(&mut self) -> LexResult<()> {
        while let Some(ch) = self.next_char()? {
            match ch {
                '\n' => break,
                _ => continue,
            }
        }

        Ok(())
    }

    fn skip_whitespace(&mut self) -> LexResult<()> {
        while let Some(ch) = self.next_char()? {
            if ch.is_whitespace() {
                continue;
            } else {
                self.back_seek()?;
                break;
            }
        }

        Ok(())
    }

    fn next_char(&mut self) -> LexResult<Option<char>> {
        let mut buf: [u8; 1] = [0];
        let read_size = self.input.read(&mut buf)?;
        if read_size >= 1 {
            Ok(Some(buf[0] as char))
        } else {
            Ok(None)
        }
    }

    fn back_seek(&mut self) -> LexResult<u64> {
        self.input
            .seek(std::io::SeekFrom::Current(-1))
            .map_err(|e| e.into())
    }
}

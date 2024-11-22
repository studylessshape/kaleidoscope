use std::mem;

use crate::{
    ast::*,
    error::ParserError,
    lex::{Lexer, Token},
    Result,
};

pub struct Parser<S>
where
    S: AsRef<[u8]>,
{
    inner_lex: Lexer<S>,
    peek: Token,
}

impl<S> Parser<S>
where
    S: AsRef<[u8]>,
{
    pub fn new(input: S) -> Result<Self> {
        let mut lex = Lexer::new(input);
        let peek = lex.next()?;
        Ok(Self {
            inner_lex: lex,
            peek,
        })
    }

    pub fn peek(&self) -> &Token {
        &self.peek
    }

    fn pop(&mut self) -> Result<Token> {
        Ok(mem::replace(&mut self.peek, self.inner_lex.next()?))
    }

    /// ```BNF
    /// expression
    ///     ::= primary binoprhs
    /// ```
    ///
    /// - primary => [`Parser::parse_primary`]
    /// - binoprhs => [`Parser::parse_binop_rhs`]
    pub fn parse_expr(&mut self) -> Result<ExprAst> {
        let lhs = self.parse_primary()?;

        self.parse_binop_rhs(0, lhs)
    }

    /// ```BNF
    /// binoprhs
    ///     ::= ('+' primary)*
    /// ```
    ///
    /// - primary => [`Parser::parse_primary`]
    pub fn parse_binop_rhs(&mut self, expr_precedence: i8, mut lhs: ExprAst) -> Result<ExprAst> {
        loop {
            let tok_prec = self.peek().precedence();
            if tok_prec < expr_precedence {
                return Ok(lhs);
            }

            let op: OpSymbol = self.pop()?.try_into()?;
            let mut rhs = self.parse_primary()?;

            let next_prec = self.peek().precedence();
            if tok_prec < next_prec {
                rhs = self.parse_binop_rhs(tok_prec + 1, rhs)?;
            }

            lhs = ExprAst::Binary(Box::new(BinaryExprAst::new(op, lhs, rhs)));
        }
    }

    /// ```BNF
    /// numberexpr ::= number
    /// ```
    ///
    /// - number => [`Token::Number`]
    pub fn parse_number(&mut self) -> Result<ExprAst> {
        let token = self.pop()?;
        if let Token::Number(number) = token {
            Ok(ExprAst::Number(number))
        } else {
            Err(ParserError::SyntaxError(format!("Expects {{Number}}, get `{token:?}`")).into())
        }
    }

    /// ```BNF
    /// parentexpr
    ///     ::= '(' expression ')'
    /// ```
    ///
    /// - expression => [`Parser::parse_expr`]
    pub fn parse_parent(&mut self) -> Result<ExprAst> {
        self.pop()?;
        let expr = self.parse_expr()?;

        match self.pop()? {
            Token::RightBracket => Ok(expr),
            _ => Err(ParserError::SyntaxError("Expects `)`".to_string()).into()),
        }
    }

    /// ```BNF
    /// identifierexpr
    ///     ::= identifier
    ///     ::= identifier '(' expression* ')'
    /// ```
    ///
    /// - identifier => [`Token::Identifier`]
    /// - expression => [`Parser::parse_expr`]
    pub fn parse_identifier(&mut self) -> Result<ExprAst> {
        let token = self.pop()?;
        if let Token::Identifier(identifier) = token {
            if let Token::LeftBracket = self.peek() {
                // eat `(`
                self.pop()?;

                let mut args = Vec::new();

                match self.peek() {
                    // eat `)`
                    Token::RightBracket => _ = self.pop()?,
                    _ => loop {
                        args.push(self.parse_expr()?);

                        let peek = self.peek();

                        if let Token::RightBracket = peek {
                            break;
                        }

                        if *peek != Token::Comma {
                            return ParserError::syn_err("Expected ')' or ',' in argument list");
                        }

                        self.pop()?;
                    },
                }

                Ok(ExprAst::Call(Box::new(CallExprAst::new(identifier, args))))
            } else {
                Ok(ExprAst::Variable(identifier))
            }
        } else {
            ParserError::syn_err(format!("Expect `identifier`, but get token: {token:?}"))
        }
    }

    /// ```BNF
    /// primary
    ///     ::= identifierexpr
    ///     ::= numberexpr
    ///     ::= parentexpr
    /// ```
    ///
    /// - identifierexpr => [`Parser::parse_identifier`]
    /// - numberexpr => [`Parser::parse_number`]
    /// - parentexpr => [`Parser::parse_parent`]
    pub fn parse_primary(&mut self) -> Result<ExprAst> {
        match self.peek() {
            Token::LeftBracket => self.parse_parent(),
            Token::Number(_) => self.parse_number(),
            Token::Identifier(_) => self.parse_identifier(),
            _ => Err(ParserError::UnexpectedToken(self.peek().clone()).into()),
        }
    }

    /// ```BNF
    /// prototype
    ///     ::= id '(' id* ')'
    /// ```
    ///
    /// - id => [`Token::Identifier`]
    pub fn parse_prototype(&mut self) -> Result<PrototypeAst> {
        if let Token::Identifier(fn_name) = self.pop()? {
            if &Token::LeftBracket != self.peek() {
                return ParserError::syn_err("Expected '(' in prototype");
            }
            // eat '('
            self.pop()?;

            let mut args = Vec::new();

            while let Token::Identifier(_) = self.peek() {
                if let Token::Identifier(arg_name) = self.pop()? {
                    args.push(arg_name);
                }
            }

            if &Token::RightBracket != self.peek() {
                return ParserError::syn_err("Expected ')' in prototype");
            }

            self.pop()?;

            Ok(PrototypeAst {
                name: fn_name,
                args,
            })
        } else {
            Err(ParserError::ExpectedFunctionName.into())
        }
    }

    /// ```BNF
    /// definition ::= 'def' prototype expression
    /// ```
    ///
    /// - 'def' => [`Token::Def`]
    /// - prototype => [`Parser::parse_prototype`]
    /// - expression => [`Parser::parse_expr`]
    pub fn parse_definition(&mut self) -> Result<FunctionAst> {
        self.pop()?; // eat def

        Ok(FunctionAst {
            proto: self.parse_prototype()?,
            body: self.parse_expr()?,
        })
    }

    /// ```BNF
    /// external ::= 'extern' prototype
    /// ```
    ///
    /// - 'extern' => [`Token::Extern`]
    /// - prototype => [`Parser::parse_prototype`]
    pub fn parse_extern(&mut self) -> Result<PrototypeAst> {
        self.pop()?;

        self.parse_prototype()
    }

    /// ```BNF
    /// toplevelexpr ::= expression
    /// ```
    ///
    /// - expression => [`Parser::parse_expr`]
    pub fn parse_toplevel(&mut self) -> Result<FunctionAst> {
        let expr = self.parse_expr()?;

        let proto = PrototypeAst {
            name: "".to_string(),
            args: Vec::new(),
        };

        Ok(FunctionAst { proto, body: expr })
    }
}

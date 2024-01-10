use super::lexer::Lexer;
use super::token::*;

use crate::ir::argtype::Type;
use crate::ir::function::{Argument, Function};
use crate::ir::interface::Interface;

pub struct Parser<'a> {
    lexer: Lexer<'a>,
}

impl<'a> Parser<'a> {
    pub fn new(lexer: Lexer<'a>) -> Self {
        Self { lexer }
    }

    fn consume_token_pred<F: Fn(&Token) -> bool>(&mut self, f: F) -> Option<Token> {
        let t = self.lexer.next()?;

        if f(&t) {
            Some(t)
        } else {
            None
        }
    }

    fn consume_token_type(&mut self, t: TokenType) -> Option<Token> {
        self.consume_token_pred(|token| token.get_type() == t)
    }

    fn consume_token(&mut self) -> Option<Token> {
        self.lexer.next_token()
    }

    fn parse_function_arg(&mut self) -> Option<Argument> {
        let arg_dir = self.consume_token_pred(|t| {
            t.get_type() == TokenType::TokenId(IdType::In)
                || t.get_type() == TokenType::TokenId(IdType::Out)
        })?;
        let arg_type = self.consume_token_type(TokenType::TokenId(IdType::Identifier))?;
        let name = self.consume_token_type(TokenType::TokenId(IdType::Identifier))?;

        if arg_dir.get_type() == TokenType::TokenId(IdType::In) {
            Some(Argument::In(
                Type::new(arg_type.get_str().to_owned()),
                name.get_str().to_owned(),
            ))
        } else {
            Some(Argument::Out(
                Type::new(arg_type.get_str().to_owned()),
                name.get_str().to_owned(),
            ))
        }
    }

    fn parse_function(&mut self, name: Token) -> Option<Function> {
        enum States {
            Start,
            ArgEnd,
            FuncEnd,
        }

        let mut func = Function::new(name.get_str().as_bytes());
        let mut state = States::Start;

        loop {
            match state {
                States::Start => {
                    self.consume_token_type(TokenType::LeftParen)?;
                    state = States::ArgEnd;
                }
                States::ArgEnd => {
                    match self.consume_token()?.get_type() {
                        TokenType::Comma => continue,
                        TokenType::RightParen => {
                            state = States::FuncEnd;
                            continue;
                        }
                        _ => {}
                    };

                    self.lexer.undo_next_token();

                    let arg = self.parse_function_arg()?;
                    func.add_arg(arg);
                }
                States::FuncEnd => {
                    self.consume_token_type(TokenType::Semicolumn)?;
                    return Some(func);
                }
            }
        }
    }

    fn parse_interface(&mut self) -> Option<Interface> {
        let mut interface = Interface::new();

        self.consume_token_type(TokenType::LeftCurlParen)?;

        loop {
            match self.consume_token() {
                Some(token) => match token.get_type() {
                    TokenType::TokenId(IdType::Identifier) => {
                        interface.add_func(self.parse_function(token)?);
                        Some(())
                    }
                    TokenType::RightCurlParen => {
                        return Some(interface);
                    }
                    _ => {
                        println!("Unxpected end of line!");
                        None
                    }
                },
                None => {
                    println!("Unxpected end of file!");
                    None
                }
            }?
        }

        // Some(interface)
    }

    pub fn parse(&mut self) -> bool {
        let t = self.consume_token_pred(|t| t.get_type() == TokenType::TokenId(IdType::Interface));

        match t {
            Some(_) => self.parse_interface().is_some(),
            None => {
                error!("Failed to parse!");
                false
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_empty_interface() {
        let text = "interface { }";
        let lexer = Lexer::new(text.as_bytes());
        let mut parser = Parser::new(lexer);

        assert_eq!(parser.parse(), true);
    }

    #[test]
    fn test_interface_with_simple_func() {
        let text = "interface { Test(in Int a); }";
        let lexer = Lexer::new(text.as_bytes());
        let mut parser = Parser::new(lexer);

        assert_eq!(parser.parse(), true);
    }

    #[test]
    fn test_interface_with_simple_func_err() {
        let text = [
            "interface { Test(in Int a) }",
            "interface { Test(test Int a) }",
            "interface { Test(Int a) }",
            "interface { Test(out Int); }",
            "interface { Test(outInt) }",
        ];

        for i in text {
            let lexer = Lexer::new(i.as_bytes());
            let mut parser = Parser::new(lexer);

            assert_eq!(parser.parse(), false);
        }
    }
}

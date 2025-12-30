use super::lexer::Lexer;
use super::token::*;
use std::collections::HashMap;

use crate::ast::argtype::{Struct, Type};
use crate::ast::function::{Argument, Function};
use crate::ast::interface::Interface;
use crate::ast::module::Module;
use crate::error_reporter::ErrorReporter;

pub struct Parser<'a> {
    lexer: Lexer<'a>,
    reporter: &'a ErrorReporter<'a>,
    aliases: HashMap<String, Type>,
    custom_types: HashMap<String, Type>,
    lookahead: Option<Token>,
}

impl<'a> Parser<'a> {
    pub fn new(lexer: Lexer<'a>, reporter: &'a ErrorReporter) -> Self {
        Self {
            lexer,
            reporter,
            aliases: HashMap::new(),
            custom_types: HashMap::new(),
            lookahead: None,
        }
    }

    fn lookahead_token_type(&mut self, t: TokenType) -> Option<Token> {
        self.lookahead_token_pred(|token| token.get_type() == t)
    }

    fn lookahead_token_pred<F: Fn(&Token) -> bool>(&mut self, f: F) -> Option<Token> {
        if let Some(la) = self.lookahead.as_ref() {
            return f(la).then_some(self.lookahead.take().unwrap());
        }

        let t = self.lexer.next()?;

        if f(&t) {
            Some(t)
        } else {
            assert!(self.lookahead.is_none());

            self.lookahead = Some(t);
            None
        }
    }

    fn consume_token_pred<F: Fn(&Token) -> bool>(&mut self, f: F) -> Option<Token> {
        let t = self.lookahead.take().or_else(|| self.lexer.next())?;

        if f(&t) {
            Some(t)
        } else {
            crate::token_or_report!(None::<Token>, self.reporter, t.clone());
            None
        }
    }

    fn consume_token_type(&mut self, t: TokenType) -> Option<Token> {
        self.consume_token_pred(|token| token.get_type() == t)
    }

    fn consume_token(&mut self) -> Option<Token> {
        self.lexer.next_token()
    }

    fn peek_token(&mut self) -> Option<Token> {
        if let Some(la) = self.lookahead.as_ref() {
            Some(la.clone())
        } else {
            let next = self.consume_token()?;

            self.lookahead = Some(next.clone());
            Some(next)
        }
    }

    fn parse_type(&mut self) -> Option<Type> {
        if let Some(tp) = self.lookahead_token_type(TokenType::TokenId(IdType::Identifier)) {
            let name = tp.get_str().to_owned();

            // There could be recursive aliases... Don't care for now
            self.aliases
                .get(&name)
                .or(self
                    .custom_types
                    .get(&name)
                    .or(Type::new(name.clone()).as_ref()))
                .cloned()
        } else if self
            .lookahead_token_type(TokenType::TokenId(IdType::Sequence))
            .is_some()
        {
            self.consume_token_type(TokenType::Less)?;

            let inner = Box::new(self.parse_type()?);
            self.consume_token_type(TokenType::Comma)?;
            let count =
                self.consume_token_pred(|x| matches!(x.get_type(), TokenType::Number(_)))?;
            let count = match count.get_type() {
                TokenType::Number(x) => x,
                _ => panic!(""),
            }
            .try_into()
            .unwrap();
            self.consume_token_type(TokenType::Greater)?;

            Some(Type::Sequence { inner, count })
        } else {
            println!("Failed to parse type!");
            None
        }
    }

    fn parse_function_arg(&mut self) -> Option<Argument> {
        let arg_dir = self.consume_token_pred(|t| {
            t.get_type() == TokenType::TokenId(IdType::In)
                || t.get_type() == TokenType::TokenId(IdType::Out)
        })?;
        let arg_type = self.parse_type()?;
        let name = self.consume_token_type(TokenType::TokenId(IdType::Identifier))?;

        if arg_dir.get_type() == TokenType::TokenId(IdType::In) {
            Some(Argument::In(arg_type, name.get_str().to_owned()))
        } else {
            Some(Argument::Out(arg_type, name.get_str().to_owned()))
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
        self.consume_token_type(TokenType::TokenId(IdType::Interface))
            .unwrap();

        let name = self.consume_token_type(TokenType::TokenId(IdType::Identifier))?;
        let mut interface = Interface::new(name.get_str().to_owned());

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
    }

    fn parse_aliase(&mut self) -> Option<()> {
        self.consume_token_type(TokenType::TokenId(IdType::Type))
            .unwrap();

        let name = self.consume_token_type(TokenType::TokenId(IdType::Identifier))?;

        self.consume_token_type(TokenType::Equal)?;
        let tp = self.parse_type()?;

        self.consume_token_type(TokenType::Semicolumn)?;
        self.aliases.insert(name.get_str().to_owned(), tp);

        Some(())
    }

    fn parse_struct(&mut self) -> Option<(String, Type)> {
        self.consume_token_type(TokenType::TokenId(IdType::Struct))
            .unwrap();

        let mut data = vec![];
        let name = self.consume_token_type(TokenType::TokenId(IdType::Identifier))?;
        self.consume_token_type(TokenType::LeftCurlParen)?;

        while self
            .lookahead_token_type(TokenType::RightCurlParen)
            .is_none()
        {
            let tp = self.parse_type()?;
            let name = self.consume_token_type(TokenType::TokenId(IdType::Identifier))?;

            self.consume_token_type(TokenType::Semicolumn)?;
            data.push((name.get_str().to_owned(), tp));
        }

        let name = name.get_str().to_owned();
        Some((name.clone(), Type::Struct(Struct { name, data })))
    }

    pub fn parse(mut self) -> Option<Module> {
        let mut mods = vec![];

        while let Some(token) = self.peek_token() {
            match token.get_type() {
                TokenType::TokenId(IdType::Type) => self.parse_aliase()?,
                TokenType::TokenId(IdType::Interface) => mods.push(self.parse_interface()?),
                TokenType::TokenId(IdType::Struct) => {
                    let (name, tp) = self.parse_struct().unwrap();
                    self.custom_types.insert(name, tp);
                }
                _ => panic!("{:?}", token),
            }
        }

        Some(Module::new(
            mods,
            self.custom_types
                .into_values()
                .map(|x| {
                    let Type::Struct(s) = x else {
                        panic!("");
                    };
                    s
                })
                .collect(),
        ))
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::ast::argtype::BuiltinTypes;
    use crate::error_reporter;

    #[test]
    fn test_empty_interface() {
        let text = "interface test { }";
        let lexer = Lexer::new(text.as_bytes());
        let reporter = error_reporter::ErrorReporter::new(text.as_bytes());
        let parser = Parser::new(lexer, &reporter);

        assert!(parser.parse().is_some());
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
            let reporter = error_reporter::ErrorReporter::new(i.as_bytes());
            let parser = Parser::new(lexer, &reporter);

            assert!(parser.parse().is_none());
        }
    }

    #[test]
    fn test_interface_with_simple_func() {
        let text = "interface test { Test(out I32 a); }";
        let lexer = Lexer::new(text.as_bytes());
        let reporter = error_reporter::ErrorReporter::new(text.as_bytes());
        let parser = Parser::new(lexer, &reporter);

        assert!(parser.parse().is_some());
    }

    #[test]
    fn test_sequence() {
        let text = "type Name = Sequence<I32, 10>;";
        let lexer = Lexer::new(text.as_bytes());
        let reporter = error_reporter::ErrorReporter::new(text.as_bytes());
        let parser = Parser::new(lexer, &reporter);

        assert!(parser.parse().is_some());
    }

    #[test]
    fn test_aliases() {
        let text = "type Name = I32; interface test { Test(out Name a); }";
        let lexer = Lexer::new(text.as_bytes());
        let reporter = error_reporter::ErrorReporter::new(text.as_bytes());
        let parser = Parser::new(lexer, &reporter);

        let md = parser.parse().unwrap();
        let arg = &md.interfaces()[0].functions()[0].args()[0];

        // Check that alias is resolved
        assert!(matches!(
            arg,
            Argument::Out(Type::Builtin(BuiltinTypes::I32), _)
        ));
    }
}

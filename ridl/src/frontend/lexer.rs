use super::token::*;

#[derive(Debug)]
pub struct Lexer<'a> {
    source: &'a [u8],
    parsed: usize,

    token_start: Option<usize>,
    prev_token: Option<usize>,
}

impl<'a> Lexer<'a> {
    pub fn new(source: &'a [u8]) -> Self {
        Self {
            source,
            parsed: 0,
            token_start: None,
            prev_token: None,
        }
    }

    fn finish_token(&mut self) -> &[u8] {
        let start = self.token_start.unwrap();

        self.reset_token();

        self.prev_token = Some(start);
        &self.source[start..self.parsed]
    }

    fn reset_token(&mut self) {
        self.token_start = None;
    }

    fn peek(&self) -> Option<u8> {
        self.source.get(self.parsed - 1).copied()
    }

    fn consume(&mut self) -> Option<u8> {
        self.parsed += 1;
        self.peek()
    }

    fn start_token(&mut self) -> Option<u8> {
        if self.token_start.is_none() {
            self.token_start = Some(self.parsed);
        }

        self.consume()
    }

    fn unconsume(&mut self) {
        self.parsed -= 1;
    }

    fn skip_whitespaces(&mut self) {
        while {
            let c = self.consume();
            if let Some(c) = c {
                if !c.is_ascii_whitespace() {
                    self.unconsume();
                    false
                } else {
                    true
                }
            } else {
                false
            }
        } {}
    }

    fn consume_word(&mut self) -> Option<Token> {
        while let Some(s) = self.peek() {
            if s.is_ascii_alphabetic() {
                self.consume();
            } else {
                self.unconsume();
                break;
            }
        }

        Some(Token::new_id(self.finish_token()))
    }

    pub fn into_iter(self) -> Self {
        self
    }

    pub fn undo_next_token(&mut self) {
        self.parsed = self.prev_token.unwrap();
    }

    pub fn next_token(&mut self) -> Option<Token> {
        self.skip_whitespaces();

        match self.start_token() {
            Some(c) => {
                match c {
                    b'{' => return Some(Token::new(TokenType::LeftCurlParen, self.finish_token())),
                    b'}' => {
                        return Some(Token::new(TokenType::RightCurlParen, self.finish_token()))
                    }
                    b'(' => {
                        return Some(Token::new(TokenType::LeftParen, self.finish_token()))
                    }
                    b')' => {
                        return Some(Token::new(TokenType::RightParen, self.finish_token()))
                    }
                    b',' => {
                        return Some(Token::new(TokenType::Comma, self.finish_token()))
                    }
                    b';' => {
                        return Some(Token::new(TokenType::Semicolumn, self.finish_token()))
                    }
                    other => {
                        if other.is_ascii_alphabetic() {
                            return self.consume_word();
                        } else if other.is_ascii_whitespace() {
                            panic!("Should be skipped already");
                        } else {
                            return None;
                        }
                    }
                };
            }
            _ => return None,
        }
    }
}

impl Iterator for Lexer<'_> {
    type Item = Token;

    fn next(&mut self) -> Option<Self::Item> {
        self.next_token()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_basic_interface() {
        let text = "interface { }";
        let lexer = Lexer::new(text.as_bytes());
        let expected = vec![
            Token::new_id("interface".as_bytes()),
            Token::new(TokenType::LeftCurlParen, "{".as_bytes()),
            Token::new(TokenType::RightCurlParen, "}".as_bytes()),
        ];

        assert_eq!(lexer.into_iter().collect::<Vec<_>>(), expected);
    }

    #[test]
    fn test_basic_interface_newline() {
        let text = "interface {\n}";
        let lexer = Lexer::new(text.as_bytes());
        let expected = vec![
            Token::new_id("interface".as_bytes()),
            Token::new(TokenType::LeftCurlParen, "{".as_bytes()),
            Token::new(TokenType::RightCurlParen, "}".as_bytes()),
        ];

        assert_eq!(lexer.into_iter().collect::<Vec<_>>(), expected);
    }

    #[test]
    fn test_basic_function() {
        let text = "interface { Test(); }";
        let lexer = Lexer::new(text.as_bytes());
        let expected = vec![
            Token::new_id("interface".as_bytes()),
            Token::new(TokenType::LeftCurlParen, "{".as_bytes()),
            Token::new(TokenType::TokenId(IdType::Identifier), "Test".as_bytes()),
            Token::new(TokenType::LeftParen, "(".as_bytes()),
            Token::new(TokenType::RightParen, ")".as_bytes()),
            Token::new(TokenType::Semicolumn, ";".as_bytes()),
            Token::new(TokenType::RightCurlParen, "}".as_bytes()),
        ];

        assert_eq!(lexer.into_iter().collect::<Vec<_>>(), expected);
    }

    #[test]
    fn test_basic_function_with_one_arg() {
        let text = "interface { Test(in Int a); }";
        let lexer = Lexer::new(text.as_bytes());
        let expected = vec![
            Token::new_id("interface".as_bytes()),
            Token::new(TokenType::LeftCurlParen, "{".as_bytes()),
            Token::new_id("Test".as_bytes()),
            Token::new(TokenType::LeftParen, "(".as_bytes()),
            Token::new_id("in".as_bytes()),
            Token::new_id("Int".as_bytes()),
            Token::new_id("a".as_bytes()),
            Token::new(TokenType::RightParen, ")".as_bytes()),
            Token::new(TokenType::Semicolumn, ";".as_bytes()),
            Token::new(TokenType::RightCurlParen, "}".as_bytes()),
        ];

        assert_eq!(lexer.into_iter().collect::<Vec<_>>(), expected);
    }

    #[test]
    fn test_basic_function_with_two_arg() {
        let text = "interface { Test(in Int a, in Int b); }";
        let lexer = Lexer::new(text.as_bytes());
        let expected = vec![
            Token::new_id("interface".as_bytes()),
            Token::new(TokenType::LeftCurlParen, "{".as_bytes()),
            Token::new_id("Test".as_bytes()),
            Token::new(TokenType::LeftParen, "(".as_bytes()),
            Token::new_id("in".as_bytes()),
            Token::new_id("Int".as_bytes()),
            Token::new_id("a".as_bytes()),
            Token::new(TokenType::Comma, ",".as_bytes()),
            Token::new_id("in".as_bytes()),
            Token::new_id("Int".as_bytes()),
            Token::new_id("b".as_bytes()),
            Token::new(TokenType::RightParen, ")".as_bytes()),
            Token::new(TokenType::Semicolumn, ";".as_bytes()),
            Token::new(TokenType::RightCurlParen, "}".as_bytes()),
        ];

        assert_eq!(lexer.into_iter().collect::<Vec<_>>(), expected);
    }
}

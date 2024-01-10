use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IdType {
    Identifier,
    Interface,
    In,
    Out,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum TokenType {
    TokenId(IdType),
    LeftCurlParen,
    RightCurlParen,
    LeftParen,
    RightParen,
    Comma,
    Semicolumn,
}

#[derive(Debug, PartialEq, Eq)]
pub struct Token {
    tp: TokenType,
    string: String,
}

lazy_static::lazy_static! {
    static ref KEYWORDS: HashMap<&'static str, TokenType> =
        HashMap::from([
            ("interface", TokenType::TokenId(IdType::Interface)),
            ("in", TokenType::TokenId(IdType::In)),
            ("in", TokenType::TokenId(IdType::Out)),
        ]);
}

impl Token {
    pub fn new_id(string: &[u8]) -> Self {
        let string = std::str::from_utf8(string).expect("Non utf8 source???");
        let tp = if let Some(id) = KEYWORDS.get(&string) {
            *id
        } else {
            TokenType::TokenId(IdType::Identifier)
        };

        Self {
            tp,
            string: string.to_owned(),
        }
    }

    pub fn new(tp: TokenType, string: &[u8]) -> Self {
        Self {
            tp,
            string: std::str::from_utf8(string)
                .expect("Non utf8 source???")
                .to_owned(),
        }
    }

    pub fn get_type(&self) -> TokenType {
        self.tp
    }

    pub fn get_str(&self) -> &str {
        self.string.as_str()
    }
}

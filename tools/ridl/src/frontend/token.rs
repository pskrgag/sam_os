use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IdType {
    Identifier,
    Interface,
    Type,
    In,
    Out,
    Sequence,
    Struct,
    Package,
    Enum,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum TokenType {
    TokenId(IdType),
    LeftCurlParen,
    RightCurlParen,
    LeftParen,
    RightParen,
    Less,
    Greater,
    Comma,
    Equal,
    Colon,
    Semicolumn,
    Number(i64),
}

#[derive(Debug, Copy, Clone, Default)]
pub struct Location {
    pub line: usize,
    pub pos: usize,
}

#[derive(Debug, Clone)]
pub struct Token {
    tp: TokenType,
    string: String,
    loc: Location,
}

lazy_static::lazy_static! {
    static ref KEYWORDS: HashMap<&'static str, TokenType> =
        HashMap::from([
            ("interface", TokenType::TokenId(IdType::Interface)),
            ("in", TokenType::TokenId(IdType::In)),
            ("out", TokenType::TokenId(IdType::Out)),
            ("type", TokenType::TokenId(IdType::Type)),
            ("Sequence", TokenType::TokenId(IdType::Sequence)),
            ("struct", TokenType::TokenId(IdType::Struct)),
            ("package", TokenType::TokenId(IdType::Package)),
            ("enum", TokenType::TokenId(IdType::Enum)),
        ]);
}

impl Token {
    pub fn new_id(string: &[u8], loc: Location) -> Self {
        let string = std::str::from_utf8(string).expect("Non utf8 source???");
        let tp = if let Some(id) = KEYWORDS.get(&string) {
            *id
        } else {
            TokenType::TokenId(IdType::Identifier)
        };

        Self {
            tp,
            string: string.to_owned(),
            loc,
        }
    }

    pub fn new_number(string: &[u8], loc: Location) -> Self {
        let string = std::str::from_utf8(string)
            .expect("Non utf8 source???")
            .to_owned();
        let num = string.parse().unwrap();

        Self {
            tp: TokenType::Number(num),
            string,
            loc,
        }
    }

    pub fn new(tp: TokenType, string: &[u8], loc: Location) -> Self {
        Self {
            tp,
            string: std::str::from_utf8(string)
                .expect("Non utf8 source???")
                .to_owned(),
            loc,
        }
    }

    pub fn get_type(&self) -> TokenType {
        self.tp
    }

    pub fn get_str(&self) -> &str {
        self.string.as_str()
    }

    pub fn location(&self) -> Location {
        self.loc
    }
}

impl PartialEq for Token {
    fn eq(&self, other: &Self) -> bool {
        self.tp == other.tp && self.string == other.string
    }
}

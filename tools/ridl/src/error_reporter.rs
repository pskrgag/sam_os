use crate::frontend::token::Token;

pub enum ErrorKind {
    UnxpectedToken(Token),
    UnknownType(Token),
}

#[macro_export]
macro_rules! token_or_report {
    ($t:expr, $reporter:expr, $token:expr) => {
        if $t.is_none() {
            $reporter.report($crate::error_reporter::ErrorKind::UnxpectedToken($token));
            None
        } else {
            $t
        }
    };
}

#[macro_export]
macro_rules! type_or_report {
    ($t:expr, $reporter:expr, $token:expr) => {
        if $t.is_none() {
            $reporter.report($crate::error_reporter::ErrorKind::UnknownType($token));
            None
        } else {
            $t
        }
    };
}

pub struct ErrorReporter<'a> {
    lines: Vec<&'a str>,
}

impl<'a> ErrorReporter<'a> {
    pub fn new(source: &'a [u8]) -> Self {
        let str = std::str::from_utf8(source).unwrap();
        let lines = str.lines();

        Self {
            lines: lines.into_iter().collect::<Vec<_>>(),
        }
    }

    pub fn report(&self, kind: ErrorKind) {
        match kind {
            ErrorKind::UnxpectedToken(t) => {
                let loc = t.location();
                error!("Unxpected Token: {}", t.get_str());
                error!("{}", self.lines[loc.line]);
                error!("{}{}\n", " ".repeat(loc.pos), "^".repeat(t.get_str().len()));
            }
            ErrorKind::UnknownType(t) => {
                let loc = t.location();
                error!("Unxpected Type: {}", t.get_str());
                error!("{}", self.lines[loc.line]);
                error!("{}{}\n", " ".repeat(loc.pos), "^".repeat(t.get_str().len()));
            }
        }
    }
}

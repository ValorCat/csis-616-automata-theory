/// Represents a symbol of the language
#[derive(Debug, PartialEq)]
pub enum Token {
    Letter(char),        // a-z, 0-9, space
    Group(Vec<Token>),   // (...)
    Union, Star, Plus,   // |, *, +
    AnyLetter, AnyDigit  // \w, \d
}

/// Converts a raw string into a sequence of tokens
pub fn tokenize(input: &str) -> Vec<Token> {
    let mut tokens = vec![];
    let mut unmatched_parens = 0;
    let mut group_start = 0;
    let mut escaped = false;
    for (i, chr) in input.chars().enumerate() {
        match chr {
            _ if escaped => tokens.push(escape_seq(chr)),
            '(' => {
                unmatched_parens += 1;
                if unmatched_parens == 1 { group_start = i + 1 }
            },
            ')' => {
                unmatched_parens -= 1;
                if unmatched_parens == 0 { tokens.push(group(&input[group_start..i])) }
            },
            '\\' => (),
            _ => if unmatched_parens == 0 { tokens.push(token(chr)) }
        }
        escaped = chr == '\\';
    }
    tokens
}

/// Find the first index of a token
pub fn find(tokens: &[Token], query: Token) -> Option<usize> {
    tokens.iter().enumerate()
            .find(|(_, t)| **t == query)
            .map(|(i, _)| i)
}

/// Find the first instance of concatenation
pub fn find_adjacent_values(tokens: &[Token]) -> Option<(usize, usize)> {
    tokens.windows(2).enumerate()
            .find(|(_, pair)| {
                let first = pair.first().unwrap();
                let second = pair.last().unwrap();
                first.is_left_value() && second.is_value()
            }).map(|(i, _)| (i, i + 1))
}    

impl Token {
    /// Is this token a value (i.e. not an operator)?
    pub fn is_value(&self) -> bool {
        use Token::*;
        match self {
            Letter(_) | Group(_) | AnyLetter | AnyDigit => true,
            _ => false
        }
    }

    /// Is this token a left-value (i.e. not a binary or left unary operator)?
    pub fn is_left_value(&self) -> bool {
        use Token::*;
        match self {
            Union => false,
            _ => true
        }
    }
}

/// Convert a character into a token
fn token(chr: char) -> Token {
    match chr {
        '|' => Token::Union,
        '*' => Token::Star,
        '+' => Token::Plus,
        'a'..='z' | '0'..='9' | ' ' => Token::Letter(chr),
        _ => {
            eprintln!("Unrecognized character `{}`", chr);
            std::process::exit(1);
        }
    }
}

/// Parse an escape sequence
fn escape_seq(chr: char) -> Token {
    match chr {
        'w' => Token::AnyLetter,
        'd' => Token::AnyDigit,
        _ => {
            eprintln!("Unrecognized escape sequence `\\{}`", chr);
            std::process::exit(1);
        }
    }
}

/// Convert a parenthesized substring into a group token
fn group(substring: &str) -> Token {
    Token::Group(tokenize(substring))
}

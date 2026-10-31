use apl_ast::{Literal, Span};
use std::iter::Peekable;
use std::str::Chars;
use thiserror::Error;

#[derive(Debug, Error, PartialEq, Clone)]
pub enum LexError {
    #[error("Unexpected character '{0}' at {1:?}")]
    UnexpectedChar(char, Span),
    #[error("Unterminated string literal starting at {0:?}")]
    UnterminatedString(Span),
    #[error("Unterminated character literal starting at {0:?}")]
    UnterminatedChar(Span),
    #[error("Invalid number literal '{0}' at {1:?}")]
    InvalidNumber(String, Span),
}

#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
    // Keywords
    Fn,
    Let,
    Mut,
    If,
    Else,
    Match,
    Return,
    Struct,
    Enum,
    Trait,
    Impl,
    Extern,
    Async,
    Await,
    Spawn,
    Channel,
    Loop,
    While,
    For,
    In,
    Pub,
    Import,
    Package,
    Break,
    Continue,
    True,
    False,
    NoneKeyword,

    // Identifiers and Literals
    Ident(String),
    Literal(Literal),
    FStringStart, // For f"hello {name}" support

    // Operators & Delimiters
    Plus,
    Minus,
    Star,
    Slash,
    Percent,
    Assign,
    Eq,
    Neq,
    Lt,
    Lte,
    Gt,
    Gte,
    And,
    Or,
    Not,
    Ampersand,
    Pipe,
    Caret,
    Question,
    Arrow,      // ->
    FatArrow,   // =>
    ColonColon, // ::
    Colon,
    Semicolon,
    Comma,
    Dot,
    DotDot,
    DotDotDot,
    LParen,
    RParen,
    LBracket,
    RBracket,
    LBrace,
    RBrace,
    At, // @ for custom allocators or decorators

    EOF,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub kind: TokenKind,
    pub span: Span,
}

pub struct Lexer<'a> {
    _input: &'a str,
    pub file_id: usize,
    chars: Peekable<Chars<'a>>,
    pos: usize,
    line: usize,
    col: usize,
}

impl<'a> Lexer<'a> {
    pub fn new(input: &'a str, file_id: usize) -> Self {
        Self {
            _input: input,
            file_id,
            chars: input.chars().peekable(),
            pos: 0,
            line: 1,
            col: 1,
        }
    }

    fn advance(&mut self) -> Option<char> {
        let c = self.chars.next()?;
        self.pos += c.len_utf8();
        if c == '\n' {
            self.line += 1;
            self.col = 1;
        } else {
            self.col += 1;
        }
        Some(c)
    }

    fn peek(&mut self) -> Option<char> {
        self.chars.peek().copied()
    }

    fn skip_whitespace_and_comments(&mut self) {
        loop {
            match self.peek() {
                Some(' ') | Some('\t') | Some('\r') | Some('\n') => {
                    self.advance();
                }
                Some('/') => {
                    let mut temp = self.chars.clone();
                    temp.next();
                    match temp.next() {
                        Some('/') => {
                            // Single line comment
                            while let Some(c) = self.advance() {
                                if c == '\n' {
                                    break;
                                }
                            }
                        }
                        Some('*') => {
                            // Multi-line comment
                            self.advance(); // skip /
                            self.advance(); // skip *
                            let mut depth = 1;
                            while depth > 0 {
                                match self.advance() {
                                    Some('*') => {
                                        if self.peek() == Some('/') {
                                            self.advance();
                                            depth -= 1;
                                        }
                                    }
                                    Some('/') => {
                                        if self.peek() == Some('*') {
                                            self.advance();
                                            depth += 1;
                                        }
                                    }
                                    Some(_) => {}
                                    None => break,
                                }
                            }
                        }
                        _ => break,
                    }
                }
                _ => break,
            }
        }
    }

    pub fn tokenize(&mut self) -> Result<Vec<Token>, LexError> {
        let mut tokens = Vec::new();
        while let Some(token) = self.next_token()? {
            let is_eof = token.kind == TokenKind::EOF;
            tokens.push(token);
            if is_eof {
                break;
            }
        }
        Ok(tokens)
    }

    pub fn next_token(&mut self) -> Result<Option<Token>, LexError> {
        self.skip_whitespace_and_comments();
        let start_pos = self.pos;
        let start_line = self.line;
        let start_col = self.col;

        let c = match self.advance() {
            Some(c) => c,
            None => {
                return Ok(Some(Token {
                    kind: TokenKind::EOF,
                    span: Span::new(self.file_id, start_pos, start_pos, start_line, start_col),
                }));
            }
        };

        let file_id = self.file_id;
        let span = |end_pos, end_line, end_col| Span::new(file_id, start_pos, end_pos, end_line, end_col);

        let kind = match c {
            '+' => TokenKind::Plus,
            '-' => {
                if self.peek() == Some('>') {
                    self.advance();
                    TokenKind::Arrow
                } else {
                    TokenKind::Minus
                }
            }
            '*' => TokenKind::Star,
            '/' => TokenKind::Slash,
            '%' => TokenKind::Percent,
            '=' => {
                if self.peek() == Some('=') {
                    self.advance();
                    TokenKind::Eq
                } else if self.peek() == Some('>') {
                    self.advance();
                    TokenKind::FatArrow
                } else {
                    TokenKind::Assign
                }
            }
            '!' => {
                if self.peek() == Some('=') {
                    self.advance();
                    TokenKind::Neq
                } else {
                    TokenKind::Not
                }
            }
            '<' => {
                if self.peek() == Some('=') {
                    self.advance();
                    TokenKind::Lte
                } else {
                    TokenKind::Lt
                }
            }
            '>' => {
                if self.peek() == Some('=') {
                    self.advance();
                    TokenKind::Gte
                } else {
                    TokenKind::Gt
                }
            }
            '&' => {
                if self.peek() == Some('&') {
                    self.advance();
                    TokenKind::And
                } else {
                    TokenKind::Ampersand
                }
            }
            '|' => {
                if self.peek() == Some('|') {
                    self.advance();
                    TokenKind::Or
                } else {
                    TokenKind::Pipe
                }
            }
            '^' => TokenKind::Caret,
            '?' => TokenKind::Question,
            ':' => {
                if self.peek() == Some(':') {
                    self.advance();
                    TokenKind::ColonColon
                } else {
                    TokenKind::Colon
                }
            }
            ';' => TokenKind::Semicolon,
            ',' => TokenKind::Comma,
            '.' => {
                if self.peek() == Some('.') {
                    self.advance();
                    if self.peek() == Some('.') {
                        self.advance();
                        TokenKind::DotDotDot
                    } else {
                        TokenKind::DotDot
                    }
                } else {
                    TokenKind::Dot
                }
            }
            '(' => TokenKind::LParen,
            ')' => TokenKind::RParen,
            '[' => TokenKind::LBracket,
            ']' => TokenKind::RBracket,
            '{' => TokenKind::LBrace,
            '}' => TokenKind::RBrace,
            '@' => TokenKind::At,
            '"' => {
                let mut s = String::new();
                loop {
                    match self.advance() {
                        Some('"') => break,
                        Some('\\') => match self.advance() {
                            Some('n') => s.push('\n'),
                            Some('t') => s.push('\t'),
                            Some('r') => s.push('\r'),
                            Some('\\') => s.push('\\'),
                            Some('"') => s.push('"'),
                            Some(other) => s.push(other),
                            None => {
                                return Err(LexError::UnterminatedString(span(
                                    self.pos, self.line, self.col,
                                )));
                            }
                        },
                        Some(ch) => s.push(ch),
                        None => {
                            return Err(LexError::UnterminatedString(span(
                                self.pos, self.line, self.col,
                            )));
                        }
                    }
                }
                TokenKind::Literal(Literal::String(s))
            }
            '\'' => {
                let ch = match self.advance() {
                    Some('\\') => match self.advance() {
                        Some('n') => '\n',
                        Some('t') => '\t',
                        Some('r') => '\r',
                        Some('\\') => '\\',
                        Some('\'') => '\'',
                        Some(other) => other,
                        None => {
                            return Err(LexError::UnterminatedChar(span(
                                self.pos, self.line, self.col,
                            )));
                        }
                    },
                    Some(ch) => ch,
                    None => {
                        return Err(LexError::UnterminatedChar(span(
                            self.pos, self.line, self.col,
                        )));
                    }
                };
                if self.advance() != Some('\'') {
                    return Err(LexError::UnterminatedChar(span(
                        self.pos, self.line, self.col,
                    )));
                }
                TokenKind::Literal(Literal::Char(ch))
            }
            '0'..='9' => {
                let mut num_str = c.to_string();
                let mut is_float = false;
                while let Some(ch) = self.peek() {
                    if ch.is_ascii_digit() || ch == '_' {
                        if ch != '_' {
                            num_str.push(ch);
                        }
                        self.advance();
                    } else if ch == '.' {
                        // Check if next character after . is a digit (not method call or range ..)
                        let mut temp = self.chars.clone();
                        temp.next(); // consume .
                        if let Some(next_ch) = temp.next() {
                            if next_ch.is_ascii_digit() && !is_float {
                                is_float = true;
                                num_str.push('.');
                                self.advance();
                            } else {
                                break;
                            }
                        } else {
                            break;
                        }
                    } else {
                        break;
                    }
                }
                if is_float {
                    match num_str.parse::<f64>() {
                        Ok(f) => TokenKind::Literal(Literal::Float(f)),
                        Err(_) => {
                            return Err(LexError::InvalidNumber(
                                num_str,
                                span(self.pos, self.line, self.col),
                            ));
                        }
                    }
                } else {
                    match num_str.parse::<i64>() {
                        Ok(i) => TokenKind::Literal(Literal::Int(i)),
                        Err(_) => {
                            return Err(LexError::InvalidNumber(
                                num_str,
                                span(self.pos, self.line, self.col),
                            ));
                        }
                    }
                }
            }
            'a'..='z' | 'A'..='Z' | '_' => {
                let mut ident = c.to_string();
                while let Some(ch) = self.peek() {
                    if ch.is_ascii_alphanumeric() || ch == '_' {
                        ident.push(ch);
                        self.advance();
                    } else {
                        break;
                    }
                }
                match ident.as_str() {
                    "fn" => TokenKind::Fn,
                    "let" => TokenKind::Let,
                    "mut" => TokenKind::Mut,
                    "if" => TokenKind::If,
                    "else" => TokenKind::Else,
                    "match" => TokenKind::Match,
                    "return" => TokenKind::Return,
                    "struct" => TokenKind::Struct,
                    "enum" => TokenKind::Enum,
                    "trait" => TokenKind::Trait,
                    "impl" => TokenKind::Impl,
                    "extern" => TokenKind::Extern,
                    "async" => TokenKind::Async,
                    "await" => TokenKind::Await,
                    "spawn" => TokenKind::Spawn,
                    "channel" => TokenKind::Channel,
                    "loop" => TokenKind::Loop,
                    "while" => TokenKind::While,
                    "for" => TokenKind::For,
                    "in" => TokenKind::In,
                    "pub" => TokenKind::Pub,
                    "import" => TokenKind::Import,
                    "package" => TokenKind::Package,
                    "break" => TokenKind::Break,
                    "continue" => TokenKind::Continue,
                    "true" => TokenKind::Literal(Literal::Bool(true)),
                    "false" => TokenKind::Literal(Literal::Bool(false)),
                    "None" => TokenKind::NoneKeyword,
                    "f" if self.peek() == Some('"') => {
                        self.advance(); // consume "
                        // For basic f"string", let's treat it as String literal or FString start
                        // Let's parse until unescaped quote for now
                        let mut s = String::new();
                        loop {
                            match self.advance() {
                                Some('"') => break,
                                Some('\\') => match self.advance() {
                                    Some('n') => s.push('\n'),
                                    Some('t') => s.push('\t'),
                                    Some('r') => s.push('\r'),
                                    Some('\\') => s.push('\\'),
                                    Some('"') => s.push('"'),
                                    Some(other) => s.push(other),
                                    None => {
                                        return Err(LexError::UnterminatedString(span(
                                            self.pos, self.line, self.col,
                                        )));
                                    }
                                },
                                Some(ch) => s.push(ch),
                                None => {
                                    return Err(LexError::UnterminatedString(span(
                                        self.pos, self.line, self.col,
                                    )));
                                }
                            }
                        }
                        TokenKind::Literal(Literal::String(s))
                    }
                    _ => TokenKind::Ident(ident),
                }
            }
            other => {
                return Err(LexError::UnexpectedChar(
                    other,
                    span(self.pos, self.line, self.col),
                ));
            }
        };

        Ok(Some(Token {
            kind,
            span: span(self.pos, self.line, self.col),
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lexer_basic() {
        let input = "fn factorial(n: i32) -> i32 { return n * 2; }";
        let mut lexer = Lexer::new(input, 0);
        let tokens = lexer.tokenize().unwrap();
        assert_eq!(tokens[0].kind, TokenKind::Fn);
        assert_eq!(tokens[1].kind, TokenKind::Ident("factorial".to_string()));
        assert_eq!(tokens[2].kind, TokenKind::LParen);
    }
}

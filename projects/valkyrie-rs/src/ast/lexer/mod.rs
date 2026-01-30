use crate::ast::{Position, Span};

#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
    Identifier(String),
    Integer(i64),
    Float(f64),
    String(String),

    // Keywords
    Namespace,
    Using,
    Class,
    Trait,
    Imply,
    New,
    Micro,
    Macro,
    Let,
    If,
    Else,
    Match,
    Case,
    Return,
    Override,
    Mut,
    Async,
    Await,
    Yield,
    While,
    True,
    False,

    // Operators
    Plus,
    Minus,
    Star,
    Slash,
    Percent,
    Equal,        // =
    EqualEqual,   // ==
    NotEqual,     // !=
    Less,         // <
    LessEqual,    // <=
    Greater,      // >
    GreaterEqual, // >=
    Arrow,        // ->
    Bang,         // !
    Pipe,         // |
    PipePipe,     // ||
    And,          // &
    AndAnd,       // &&

    // Delimiters
    ParenL,      // (
    ParenR,      // )
    BraceL,      // {
    BraceR,      // }
    BracketL,    // [
    BracketR,    // ]
    Colon,       // :
    DoubleColon, // ::
    SemiColon,   // ;
    Comma,       // ,
    Dot,         // .
    At,          // @
    ZigZag,      // ↯

    EOF,
    Unknown(char),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub kind: TokenKind,
    pub span: Span,
}

#[derive(Clone)]
pub struct Lexer<'a> {
    chars: std::str::Chars<'a>,
    offset: usize,
    line: usize,
    column: usize,
    peeked: Option<(char, usize)>,
}

impl<'a> Lexer<'a> {
    pub fn new(src: &'a str) -> Self {
        Self { chars: src.chars(), offset: 0, line: 1, column: 1, peeked: None }
    }

    fn bump(&mut self) -> Option<(char, usize)> {
        if let Some((ch, pos)) = self.peeked.take() {
            self.offset = pos + ch.len_utf8();
            if ch == '\n' {
                self.line += 1;
                self.column = 1;
            }
            else {
                self.column += 1;
            }
            return Some((ch, pos));
        }
        let ch = self.chars.next()?;
        let pos = self.offset;
        self.offset += ch.len_utf8();

        if ch == '\n' {
            self.line += 1;
            self.column = 1;
        }
        else {
            self.column += 1;
        }

        Some((ch, pos))
    }

    fn peek(&mut self) -> Option<(char, usize)> {
        if self.peeked.is_none() {
            if let Some(ch) = self.chars.next() {
                let pos = self.offset;
                // Don't advance offset/line/column here, just store peeked
                self.peeked = Some((ch, pos));
            }
        }
        self.peeked
    }

    fn current_pos(&self) -> Position {
        Position { line: self.line, column: self.column, offset: self.offset }
    }

    fn make_span(&self, start: Position) -> Span {
        Span { start, end: self.current_pos() }
    }

    fn lex_number(&mut self, first: char, _start: Position) -> TokenKind {
        let mut s = String::new();
        s.push(first);
        while let Some((ch, _)) = self.peek() {
            if ch.is_ascii_digit() {
                let _ = self.bump(); // Just consume, we already updated pos in bump
                s.push(ch);
            }
            else if ch == '.' {
                let next = self.chars.clone().next();
                if matches!(next, Some(n) if n.is_ascii_digit()) {
                    let _ = self.bump();
                    s.push('.');
                    while let Some((d, _)) = self.peek() {
                        if d.is_ascii_digit() {
                            let _ = self.bump();
                            s.push(d);
                        }
                        else {
                            break;
                        }
                    }
                    let value = s.parse().unwrap_or(0.0);
                    return TokenKind::Float(value);
                }
                else {
                    break;
                }
            }
            else {
                break;
            }
        }
        let value = s.parse().unwrap_or(0);
        TokenKind::Integer(value)
    }

    fn lex_ident(&mut self, first: char, _start: Position) -> TokenKind {
        let mut s = String::new();
        s.push(first);
        while let Some((ch, _)) = self.peek() {
            if ch.is_alphanumeric() || ch == '_' {
                let _ = self.bump();
                s.push(ch);
            }
            else {
                break;
            }
        }
        match s.as_str() {
            "namespace" => TokenKind::Namespace,
            "using" => TokenKind::Using,
            "class" => TokenKind::Class,
            "trait" => TokenKind::Trait,
            "imply" => TokenKind::Imply,
            "new" => TokenKind::New,
            "micro" => TokenKind::Micro,
            "macro" => TokenKind::Macro,
            "let" => TokenKind::Let,
            "if" => TokenKind::If,
            "else" => TokenKind::Else,
            "match" => TokenKind::Match,
            "case" => TokenKind::Case,
            "return" => TokenKind::Return,
            "override" => TokenKind::Override,
            "mut" => TokenKind::Mut,
            "async" => TokenKind::Async,
            "await" => TokenKind::Await,
            "yield" => TokenKind::Yield,
            "while" => TokenKind::While,
            "true" => TokenKind::True,
            "false" => TokenKind::False,
            _ => TokenKind::Identifier(s),
        }
    }

    fn lex_string(&mut self, _start: Position) -> TokenKind {
        let mut s = String::new();
        while let Some((ch, _)) = self.bump() {
            if ch == '"' {
                break;
            }
            if ch == '\\' {
                if let Some((next_ch, _)) = self.bump() {
                    match next_ch {
                        'n' => s.push('\n'),
                        'r' => s.push('\r'),
                        't' => s.push('\t'),
                        '"' => s.push('"'),
                        '\\' => s.push('\\'),
                        _ => {
                            s.push('\\');
                            s.push(next_ch);
                        }
                    }
                }
                else {
                    s.push('\\');
                }
            }
            else {
                s.push(ch);
            }
        }
        TokenKind::String(s)
    }

    pub fn next_token(&mut self) -> Token {
        let (start_pos, ch) = loop {
            let start_pos = self.current_pos();
            match self.bump() {
                Some((c, _)) if c.is_whitespace() => continue,
                Some((c, _)) => break (start_pos, c),
                None => return Token { kind: TokenKind::EOF, span: Span { start: start_pos, end: start_pos } },
            }
        };

        let kind = match ch {
            '+' => TokenKind::Plus,
            '-' => {
                if let Some(('>', _)) = self.peek() {
                    self.bump();
                    TokenKind::Arrow
                }
                else {
                    TokenKind::Minus
                }
            }
            '*' => TokenKind::Star,
            '%' => TokenKind::Percent,
            '/' => {
                if let Some(('/', _)) = self.peek() {
                    // Deprecated comment style
                    eprintln!("Warning: '//' comments are deprecated, use '#' instead at {:?}", start_pos);
                    while let Some((c, _)) = self.bump() {
                        if c == '\n' {
                            break;
                        }
                    }
                    return self.next_token();
                }
                TokenKind::Slash
            }
            '#' => {
                // Comment
                while let Some((c, _)) = self.bump() {
                    if c == '\n' {
                        break;
                    }
                }
                return self.next_token();
            }
            '=' => {
                if let Some(('=', _)) = self.peek() {
                    self.bump();
                    TokenKind::EqualEqual
                }
                else {
                    TokenKind::Equal
                }
            }
            '!' => {
                if let Some(('=', _)) = self.peek() {
                    self.bump();
                    TokenKind::NotEqual
                }
                else {
                    TokenKind::Bang
                }
            }
            '|' => {
                if let Some(('|', _)) = self.peek() {
                    self.bump();
                    TokenKind::PipePipe
                }
                else {
                    TokenKind::Pipe
                }
            }
            '&' => {
                if let Some(('&', _)) = self.peek() {
                    self.bump();
                    TokenKind::AndAnd
                }
                else {
                    TokenKind::And
                }
            }
            '<' => {
                if let Some(('=', _)) = self.peek() {
                    self.bump();
                    TokenKind::LessEqual
                }
                else {
                    TokenKind::Less
                }
            }
            '>' => {
                if let Some(('=', _)) = self.peek() {
                    self.bump();
                    TokenKind::GreaterEqual
                }
                else {
                    TokenKind::Greater
                }
            }
            '(' => TokenKind::ParenL,
            ')' => TokenKind::ParenR,
            '{' => TokenKind::BraceL,
            '}' => TokenKind::BraceR,
            '[' => TokenKind::BracketL,
            ']' => TokenKind::BracketR,
            ':' => {
                if let Some((':', _)) = self.peek() {
                    self.bump();
                    TokenKind::DoubleColon
                }
                else {
                    TokenKind::Colon
                }
            }
            ';' => TokenKind::SemiColon,
            ',' => TokenKind::Comma,
            '.' => TokenKind::Dot,
            '@' => TokenKind::At,
            '↯' => TokenKind::ZigZag,
            '"' => self.lex_string(start_pos),
            c if c.is_ascii_digit() => self.lex_number(c, start_pos),
            c if c.is_alphabetic() || c == '_' => self.lex_ident(c, start_pos),
            _ => TokenKind::Unknown(ch),
        };

        let span = self.make_span(start_pos);
        Token { kind, span }
    }
}

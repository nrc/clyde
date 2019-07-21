use super::{lexer, Error};
use derive_new::new;
use std::fmt;

#[derive(new, Clone, Eq, PartialEq, Debug)]
pub struct Token {
    pub kind: TokenKind,
    pub span: Span,
}

impl Token {
    pub fn is_empty(&self) -> bool {
        match &self.kind {
            TokenKind::Tree(tt) => tt.tokens.is_empty(),
            TokenKind::RawTree => self.span.text.trim().is_empty(),
            _ => false,
        }
    }

    pub fn expect_tree(self) -> (TokenTree, Span) {
        match self.kind {
            TokenKind::Tree(tt) => (tt, self.span),
            _ => panic!("Expected token tree, found: {:?}", self),
        }
    }

    pub fn expect_raw_tree(&self) -> Result<(TokenTree, Span), Error> {
        match self.kind {
            TokenKind::RawTree => {
                let tt = lexer::lex(self.span.inner(), self.span.start + 1)?;
                Ok(tt.expect_tree())
            }
            _ => panic!("Expected token tree, found: {:?}", self),
        }
    }
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &self.kind {
            TokenKind::Symbol(s) => s.fmt(f),
            TokenKind::Ident => write!(f, "{}", self.span.text),
            TokenKind::Number(n) => n.fmt(f),
            TokenKind::RawTree | TokenKind::Tree(_) => write!(f, "("),
        }
    }
}

#[derive(Clone, Eq, PartialEq, Debug)]
pub enum TokenKind {
    Symbol(SymbolKind),
    Ident,
    Number(i64),
    // Note that the span for the token trees includes the delimiters, but no
    // padding outside the delimiters.
    RawTree,
    Tree(TokenTree),
}

#[derive(Clone, Eq, PartialEq, Debug)]
pub struct TokenTree {
    pub tokens: Vec<Token>,
}

#[derive(Clone, Copy, Eq, PartialEq, Debug)]
pub enum SymbolKind {
    Caret,
    Asterisk,
    Dollar,

    SemiColon,
    Hash,

    Eq,
    PlusEq,
    ArrowLeft,
    ArrowRight,
}

impl fmt::Display for SymbolKind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            SymbolKind::Caret => write!(f, "^"),
            SymbolKind::Asterisk => write!(f, "*"),
            SymbolKind::Dollar => write!(f, "$"),
            SymbolKind::SemiColon => write!(f, ";"),
            SymbolKind::Hash => write!(f, "#"),
            SymbolKind::Eq => write!(f, "="),
            SymbolKind::PlusEq => write!(f, "+="),
            SymbolKind::ArrowLeft => write!(f, "->"),
            SymbolKind::ArrowRight => write!(f, "<-"),
        }
    }
}

#[derive(new, Clone, Eq, PartialEq, Debug)]
pub struct Span {
    pub start: usize,
    pub text: String,
}

impl Span {
    pub fn inner(&self) -> &str {
        self.text[1..self.text.len() - 1].trim()
    }
}

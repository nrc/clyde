use derive_new::new;

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

#[derive(new, Clone, Eq, PartialEq, Debug)]
pub struct Span {
    pub start: usize,
    pub text: String,
}

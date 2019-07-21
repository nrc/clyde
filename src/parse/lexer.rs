use super::tokens::*;
use crate::parse;

pub fn lex(input: &str, offset: usize) -> Result<Token, parse::Error> {
    let mut lexer = Lexer {
        input,
        position: 0,
        offset,
    };
    lexer.lex_tree()
}

struct Lexer<'a> {
    input: &'a str,
    // The current position within input.
    position: usize,
    // The offset from the start of the logical input and the start of `input`.
    offset: usize,
}

impl<'a> Lexer<'a> {
    /// Lex a token tree from the current state.
    ///
    /// Postcondition: if result is Ok, then Token.kind is TokenTree.
    fn lex_tree(&mut self) -> Result<Token, parse::Error> {
        let mut tokens = Vec::new();
        loop {
            let current_input = &self.input[self.position..];
            if current_input.is_empty() {
                break;
            }
            match self.lex_tok()? {
                Some((t, len)) => match &t.kind {
                    TokenKind::Symbol(SymbolKind::Hash) => {
                        break;
                    }
                    TokenKind::Symbol(SymbolKind::SemiColon) => {
                        tokens.push(t);
                        self.position += len;
                        break;
                    }
                    _ => {
                        tokens.push(t);
                        self.position += len;
                    }
                },
                // Whitespace
                None => {
                    self.position += 1;
                }
            }
        }
        Ok(Token {
            kind: TokenKind::Tree(TokenTree { tokens }),
            span: Span::new(self.offset, self.input[..self.position].to_owned()),
        })
    }

    /// Lex a single token from the current input.
    ///
    /// Precondition `!self.input[self.position..].is_empty()`
    /// The returned usize is the length of the token in bytes (not chars).
    fn lex_tok(&self) -> Result<Option<(Token, usize)>, parse::Error> {
        let mut chars = self.input[self.position..].chars();
        match chars.next().unwrap() {
            '^' => Ok(Some((self.make_symbol(SymbolKind::Caret), 1))),
            '$' => Ok(Some((self.make_symbol(SymbolKind::Dollar), 1))),
            '*' => Ok(Some((self.make_symbol(SymbolKind::Asterisk), 1))),
            '=' => Ok(Some((self.make_symbol(SymbolKind::Eq), 1))),
            '#' => Ok(Some((self.make_symbol(SymbolKind::Hash), 1))),
            ';' => Ok(Some((self.make_symbol(SymbolKind::SemiColon), 1))),
            '-' => match chars.next() {
                None => Err(self.make_err("Unexpected end of input, expected `>`".to_owned(), 1)),
                Some('>') => Ok(Some((
                    Token::new(TokenKind::Symbol(SymbolKind::ArrowRight), self.make_span(2)),
                    2,
                ))),
                Some(_) => Err(self.make_err("Unexpected token".to_owned(), 1)),
            },
            '(' => {
                let mut len = 1;
                let mut delim_stack = vec![')'];
                loop {
                    match chars.next() {
                        Some('(') => {
                            len += 1;
                            delim_stack.push(')');
                        }
                        Some(c) if c == *delim_stack.last().unwrap() => {
                            len += 1;
                            delim_stack.pop().unwrap();
                            if delim_stack.is_empty() {
                                break;
                            }
                        }
                        Some(c) => {
                            len += c.len_utf8();
                        }
                        None => {
                            return Err(self.make_err(
                                format!(
                                    "Unexpected end of input (unclosed delimiters), expected `{}`",
                                    encode_ascii(&delim_stack)
                                ),
                                len - 1,
                            ))
                        }
                    }
                }
                Ok(Some((
                    Token::new(TokenKind::RawTree, self.make_span(len)),
                    len,
                )))
            }
            c if c.is_alphabetic() => {
                let mut len = c.len_utf8();
                loop {
                    match chars.next() {
                        Some(c) if c.is_alphanumeric() => {
                            len += c.len_utf8();
                        }
                        _ => break,
                    }
                }
                Ok(Some((
                    Token::new(TokenKind::Ident, self.make_span(len)),
                    len,
                )))
            }
            c if c.is_whitespace() => Ok(None),
            _ => Err(self.make_err("Unexpected token".to_owned(), 0)),
        }
    }

    fn make_err(&self, msg: String, offset: usize) -> parse::Error {
        parse::Error::Lexing(msg, self.offset + self.position + offset)
    }

    fn make_symbol(&self, kind: SymbolKind) -> Token {
        Token::new(TokenKind::Symbol(kind), self.make_char_span())
    }

    /// Make a Span for a single character at the current position in the input.
    fn make_char_span(&self) -> Span {
        let c = self.input[self.position..].chars().next().unwrap();
        Span::new(self.offset + self.position, c.to_string())
    }

    /// Make a Span for the `byte_len` bytes of input from the current position.
    ///
    /// Precondition: `self.position + byte_len <= self.input.len()`
    /// Precondition: the substring of `self.input` of length `byte_len` starting at `self.position`
    /// is valid utf8.
    fn make_span(&self, byte_len: usize) -> Span {
        let pos = self.position;
        let s = self.input[pos..pos + byte_len].to_owned();
        Span::new(self.offset + pos, s)
    }
}

/// Precondition: each char is one byte wide
fn encode_ascii(chars: &[char]) -> String {
    let mut result = vec![0; chars.len()];
    for (i, c) in chars.iter().enumerate() {
        c.encode_utf8(&mut result[i..]);
    }
    String::from_utf8(result).unwrap()
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn smoke() {
        assert_eq!(
            lex("", 0).unwrap(),
            Token {
                kind: TokenKind::Tree(TokenTree { tokens: Vec::new() }),
                span: Span::new(0, String::new()),
            }
        );

        assert_eq!(
            lex("   ", 0).unwrap(),
            Token {
                kind: TokenKind::Tree(TokenTree { tokens: Vec::new() }),
                span: Span::new(0, "   ".to_owned()),
            }
        );

        assert_eq!(
            lex(" $ $  ->     ", 0).unwrap(),
            Token {
                kind: TokenKind::Tree(TokenTree {
                    tokens: vec![
                        Token {
                            kind: TokenKind::Symbol(SymbolKind::Dollar),
                            span: Span::new(1, "$".to_owned())
                        },
                        Token {
                            kind: TokenKind::Symbol(SymbolKind::Dollar),
                            span: Span::new(3, "$".to_owned())
                        },
                        Token {
                            kind: TokenKind::Symbol(SymbolKind::ArrowRight),
                            span: Span::new(6, "->".to_owned())
                        },
                    ]
                }),
                span: Span::new(0, " $ $  ->     ".to_owned()),
            }
        );

        assert_eq!(
            lex("  foo  (fd && dfs: Foo( )  ) # a comment", 0).unwrap(),
            Token {
                kind: TokenKind::Tree(TokenTree {
                    tokens: vec![
                        Token {
                            kind: TokenKind::Ident,
                            span: Span::new(2, "foo".to_owned())
                        },
                        Token {
                            kind: TokenKind::RawTree,
                            span: Span::new(7, "(fd && dfs: Foo( )  )".to_owned())
                        },
                    ]
                }),
                span: Span::new(0, "  foo  (fd && dfs: Foo( )  ) ".to_owned()),
            }
        );
    }

    #[test]
    fn errors() {
        // FIXME test error messages and spans
        assert!(lex("%", 0).is_err());
        assert!(lex("-4", 0).is_err());
        assert!(lex("-", 0).is_err());
        assert!(lex("(foo", 0).is_err());
    }
}

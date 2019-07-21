use crate::parse::{self, ast, tokens, Context, Error};

pub fn parse_stmt(toks: tokens::Token, ctx: Context) -> Result<ast::Statement, Error> {
    let (tt, _) = toks.expect_tree();
    let mut parser = Parser {
        tokens: tt.tokens,
        position: 0,
        ctx,
    };
    let result = parser.parse_stmt()?;
    parser.end()?;
    Ok(result)
}

struct Parser {
    tokens: Vec<tokens::Token>,
    position: usize,
    ctx: Context,
}

impl Parser {
    fn parse_stmt(&mut self) -> Result<ast::Statement, Error> {
        let tok = match self.peek() {
            Some(tok) => tok,
            None => return Err(self.make_err("Expected statement, found ``".to_owned())),
        };
        let kind = match tok.kind {
            tokens::TokenKind::Ident => match &*tok.span.text {
                "select" => ast::StatementKind::Expr(ast::ExprKind::Select(self.select()?)),
                "show" => ast::StatementKind::Show(self.show()?),
                i => return Err(self.make_err(format!("Expected statement, found `{}`", i))),
            },
            tokens::TokenKind::Symbol(sym) => match sym {
                tokens::SymbolKind::Dollar => {
                    self.bump();
                    ast::StatementKind::Expr(ast::ExprKind::MetaVar(ast::MetaVarKind::Dollar))
                }
                _ => return Err(self.make_err(format!("Expected statement, found `{}`", sym))),
            },
            _ => return Err(self.make_err("Expected statement, TODO found what?".to_owned())),
        };
        self.maybe_semi();

        Ok(ast::Statement {
            kind,
            ctx: self.ctx.clone(),
        })
    }

    fn parse_expr(&mut self) -> Result<ast::Expr, Error> {
        self.exactly_one("expression", |this| this.maybe_expr())
    }

    fn maybe_expr(&mut self) -> Result<Option<ast::Expr>, Error> {
        let tok = match self.peek() {
            Some(tok) => tok,
            None => return Ok(None),
        };
        let kind = match tok.kind {
            tokens::TokenKind::Ident => match &*tok.span.text {
                "select" => ast::ExprKind::Select(self.select()?),
                _ => return Ok(None),
            },
            tokens::TokenKind::Symbol(sym) => match sym {
                tokens::SymbolKind::Dollar => {
                    self.bump();
                    ast::ExprKind::MetaVar(ast::MetaVarKind::Dollar)
                }
                _ => return Ok(None),
            },
            tokens::TokenKind::RawTree => {
                let inner = tok.span.inner();
                if inner.starts_with(':') {
                    let loc_parser = LocationParser::new(inner, self.ctx.clone());
                    let loc = loc_parser.location()?;
                    self.bump();
                    ast::ExprKind::Location(loc)
                } else {
                    let (tt, _) = tok.expect_raw_tree()?;
                    self.bump();
                    let mut parser = Parser {
                        tokens: tt.tokens,
                        position: 0,
                        ctx: self.ctx.clone(),
                    };
                    match parser.maybe_expr()? {
                        Some(expr) => return Ok(Some(expr)),
                        None => ast::ExprKind::Void,
                    }
                }
            }
            _ => return Ok(None),
        };

        Ok(Some(ast::Expr {
            kind,
            ctx: self.ctx.clone(),
        }))
    }

    fn select(&mut self) -> Result<ast::Select, Error> {
        self.assert_ident("select")?;

        let mut multiplicity = ast::Multiplicity::One;
        if let Some(tok) = self.peek() {
            match &tok.kind {
                tokens::TokenKind::Symbol(s) if *s == tokens::SymbolKind::Asterisk => {
                    self.bump();
                    multiplicity = ast::Multiplicity::Many;
                }
                _ => {}
            };
        }

        let filters = self.one_or_more("expression", |this| this.maybe_expr())?;

        Ok(ast::Select {
            multiplicity,
            filters,
            ctx: self.ctx.clone(),
        })
    }

    fn show(&mut self) -> Result<ast::Show, Error> {
        self.assert_ident("show")?;
        let expr = Box::new(self.parse_expr()?);
        Ok(ast::Show {
            expr,
            ctx: self.ctx.clone(),
        })
    }

    fn apply(&mut self) -> Result<ast::Apply, Error> {
        let ident = self.identifier()?;
        let args = self.one_or_more("expression", |this| this.maybe_expr())?;
        Ok(ast::Apply {
            ident,
            args,
            ctx: self.ctx.clone(),
        })
    }

    fn identifier(&mut self) -> Result<ast::Identifier, Error> {
        let next = self.next()?;
        match next.kind {
            tokens::TokenKind::Ident => {
                return Ok(ast::Identifier {
                    name: next.span.text.clone(),
                    ctx: self.ctx.clone(),
                });
            }
            _ => {}
        }

        let next = next.to_string();
        Err(self.make_err(format!("Expected identifier, found `{}`", next)))
    }

    fn maybe_semi(&mut self) -> Result<(), Error> {
        if let Some(tok) = self.peek() {
            match tok.kind {
                tokens::TokenKind::Symbol(tokens::SymbolKind::SemiColon) => {
                    self.bump();
                }
                _ => {
                    return Err(self.make_err(format!("Unexpected token: `{}`", tok)));
                }
            }
        }
        Ok(())
    }

    fn end(&self) -> Result<(), Error> {
        if self.position < self.tokens.len() {
            Err(self.make_err(format!(
                "Unexpected token: `{}`",
                self.tokens[self.position]
            )))
        } else {
            Ok(())
        }
    }

    fn peek(&self) -> Option<&tokens::Token> {
        if self.position < self.tokens.len() {
            Some(&self.tokens[self.position])
        } else {
            None
        }
    }

    fn bump(&mut self) {
        if self.position < self.tokens.len() {
            self.position += 1;
        }
    }

    fn next(&mut self) -> Result<&tokens::Token, Error> {
        if self.position < self.tokens.len() {
            let pos = self.position;
            self.bump();
            Ok(&self.tokens[pos])
        } else {
            Err(self.make_err("Unexpected end of statement".to_owned()))
        }
    }

    fn assert_ident(&mut self, s: &str) -> Result<(), Error> {
        let next = self.next()?;
        match next.kind {
            tokens::TokenKind::Ident if next.span.text == s => {
                return Ok(());
            }
            _ => {}
        }

        let next = next.to_string();
        Err(self.make_err(format!("Expected `{}`, found `{}`", s, next)))
    }

    fn zero_or_more<F, T>(&mut self, mut f: F) -> Result<Vec<T>, Error>
    where
        F: FnMut(&mut Self) -> Result<Option<T>, Error>,
    {
        let mut result = Vec::new();
        while let Some(t) = f(self)? {
            result.push(t);
        }
        Ok(result)
    }

    fn one_or_more<F, T>(&mut self, expected: &str, f: F) -> Result<Vec<T>, Error>
    where
        F: FnMut(&mut Self) -> Result<Option<T>, Error>,
    {
        let result = self.zero_or_more(f)?;
        if result.is_empty() {
            Err(self.make_err(format!("Expected {}, TODO found what?", expected)))
        } else {
            Ok(result)
        }
    }

    fn exactly_one<F, T>(&mut self, expected: &str, f: F) -> Result<T, Error>
    where
        F: FnOnce(&mut Self) -> Result<Option<T>, Error>,
    {
        match f(self)? {
            Some(t) => Ok(t),
            None => Err(self.make_err(format!("Expected {}, TODO found what?", expected))),
        }
    }

    fn make_err(&self, msg: String) -> parse::Error {
        parse::Error::Parsing(msg)
    }
}

// Parse a location.
//
// A location consists of a filename, a line number, and column number. All parts are optional.
//
// `:` unspecified location
// `:str` just a filename (which may be a pattern rather than a concrete filename)
// `:n` just a line number
// `:str:n` filename and line number
// `:n:n` line and column numbers
// `:str:n:n` fully specified
//
// Note that a trailing colon is permitted for any of the above forms.
struct LocationParser {
    input: String,
    ctx: Context,
}

impl LocationParser {
    fn new(input: &str, ctx: Context) -> LocationParser {
        LocationParser {
            input: input.to_owned(),
            ctx,
        }
    }

    fn location(self) -> Result<ast::Location, Error> {
        if !self.input.starts_with(':') {
            return Err(parse::Error::Parsing(format!(
                "Invalid location, expected `:`, found `{}`",
                self.input
            )));
        }

        let mut splits = self.input[1..].split(':');
        let first = splits.next().map(|s| s.trim());
        let second = splits.next().map(|s| s.trim());
        let third = splits.next().map(|s| s.trim());

        if let Some(s) = splits.next() {
            if !s.is_empty() {
                return Err(parse::Error::Parsing(format!(
                    "Invalid location, unexpected `{}`",
                    s
                )));
            }
        }

        match first {
            None => Ok(ast::Location::new(None, None, None, self.ctx)),
            Some(s) => match s.parse::<usize>() {
                Ok(row) => {
                    if let Some(s) = third {
                        return Err(parse::Error::Parsing(format!(
                            "Invalid location, unexpected `{}`",
                            s
                        )));
                    }
                    let second = Self::map_parse(second)?;
                    Ok(ast::Location::new(None, Some(row), second, self.ctx))
                }
                Err(_) => {
                    let second = Self::map_parse(second)?;
                    let third = Self::map_parse(third)?;
                    Ok(ast::Location::new(
                        Some(s.to_owned()),
                        second,
                        third,
                        self.ctx,
                    ))
                }
            },
        }
    }

    fn map_parse(s: Option<&str>) -> Result<Option<usize>, Error> {
        match s {
            Some(s) => match s.parse::<usize>() {
                Ok(n) => Ok(Some(n)),
                Err(_) => Err(parse::Error::Parsing(format!(
                    "Invalid location, expected number, found `{}`",
                    s
                ))),
            },
            None => Ok(None),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::parse::lexer;

    fn parser(tt: tokens::Token) -> Parser {
        Parser {
            tokens: tt.expect_tree().0.tokens,
            position: 0,
            ctx: Context::default(),
        }
    }

    #[test]
    fn smoke() {
        let toks = lexer::lex("show $;", 0).unwrap();
        parser(toks).parse_stmt().unwrap();

        let toks = lexer::lex("select* (id $)", 0).unwrap();
        parser(toks).parse_stmt().unwrap();
    }

    #[test]
    fn locations() {
        assert!(LocationParser::new("", Context::default())
            .location()
            .is_err());

        let loc = LocationParser::new(":foo.rs", Context::default())
            .location()
            .unwrap();
        assert!(loc.file.is_some() && loc.line.is_none() && loc.column.is_none());

        let loc = LocationParser::new(":0", Context::default())
            .location()
            .unwrap();
        assert!(loc.file.is_none() && loc.line.is_some() && loc.column.is_none());

        let loc = LocationParser::new(":42:3", Context::default())
            .location()
            .unwrap();
        assert!(loc.file.is_none() && loc.line.is_some() && loc.column.is_some());

        let loc = LocationParser::new(":src/bar.rs:1:2:", Context::default())
            .location()
            .unwrap();
        assert!(loc.file.is_some() && loc.line.is_some() && loc.column.is_some());
    }
}

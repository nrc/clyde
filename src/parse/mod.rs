mod ast;
mod lexer;
mod parser;
mod tokens;

#[derive(Debug, Clone)]
pub enum Error {
    // String is the error message, usize is the offset into the input.
    Lexing(String, usize),
    Parsing(String),
    EmptyInput,
    Other(String),
}

// FIXME we include this context with each node, it should include information
// specific to the node, e.g. tokens/spans
/// Contextual information about input or output to parsing.
#[derive(Default)]
pub struct Context {
    input: Option<String>,
    env_ctx: Option<Box<dyn EnvContext>>,
}

impl Clone for Context {
    fn clone(&self) -> Context {
        Context {
            input: self.input.clone(),
            env_ctx: self.env_ctx.as_ref().map(|ctx| (&**ctx).clone()),
        }
    }
}

pub trait EnvContext {
    fn clone(&self) -> Box<dyn EnvContext>;
}

pub fn parse_stmt(s: &str, env_ctx: Option<Box<dyn EnvContext>>) -> Result<ast::Statement, Error> {
    let mut ctx = Context::default();
    ctx.input = Some(s.to_owned());
    ctx.env_ctx = env_ctx;
    let toks = lexer::lex(s, 0)?;
    if toks.is_empty() {
        return Err(Error::EmptyInput);
    }
    parser::parse_stmt(toks, ctx)
}

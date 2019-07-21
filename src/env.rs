use crate::parse;

pub(crate) mod repl;

pub trait Environment {
    type ParseContext: parse::EnvContext;
}

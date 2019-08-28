pub use rls::Rls;

use crate::file_system;
use crate::front::data::{Definition, Identifier, Position, Range};
use std::fmt;

mod rls;

pub trait Backend {
    fn ident_at(&self, _position: Position) -> Result<Option<Identifier>, Error> {
        Err(Error::NotImplemented("ident_at"))
    }
    fn idents_in(&self, _range: Range) -> Result<Vec<Identifier>, Error> {
        Err(Error::NotImplemented("idents_in"))
    }
    fn definition(&self, _id: Identifier) -> Result<Definition, Error> {
        Err(Error::NotImplemented("definition"))
    }
}

pub enum Error {
    NotImplemented(&'static str),
    Back(String),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::NotImplemented(s) => {
                write!(f, "Function not implemented by current backend: `{}`", s)
            }
            Error::Back(s) => s.fmt(f),
        }
    }
}

impl From<file_system::Error> for Error {
    fn from(e: file_system::Error) -> Error {
        Error::Back(format!("file system error: {}", e))
    }
}

use crate::front;
use crate::front::data::{Position, Range};
use crate::parse::ast;
use std::fmt;

pub use self::physical::PhysicalFs;

mod physical;

pub trait FileSystem {
    // TODO should return a Result
    fn with_file<F, T>(&self, path: &Path, f: F) -> T
    where
        F: FnOnce(&File) -> T;

    fn find(&self, pat: SearchPattern) -> Result<Vec<Path>, Error>;
    fn resolve_location(&self, loc: ast::Location) -> Result<front::Locator, Error>;
}

#[derive(Clone, Eq, PartialEq)]
pub struct File {
    pub path: Path,
    pub lines: Vec<String>,
}

// TODO
#[derive(Clone, Eq, PartialEq, Hash, Debug)]
pub struct Path {}

impl fmt::Display for Path {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        unimplemented!();
    }
}

#[derive(Clone, Eq, PartialEq)]
pub struct SearchPattern {
    name: String,
}

impl From<String> for SearchPattern {
    fn from(name: String) -> SearchPattern {
        SearchPattern { name }
    }
}

#[derive(Debug, Clone)]
pub enum Error {
    Other(String),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::Other(s) => write!(f, "File error: {}", s),
        }
    }
}

// Helper function which should only be used by file systems
fn resolve_location<Fs: FileSystem>(loc: ast::Location, fs: &Fs) -> Result<front::Locator, Error> {
    // TODO resolve paths using fs.find
    match loc.file {
        Some(f) => match loc.line {
            Some(l) if l > 0 => match loc.column {
                Some(c) if c > 0 => Ok(front::Locator::Position(Position {
                    file: Path {},
                    line: l - 1,
                    column: c - 1,
                })),
                _ => Ok(front::Locator::Range(Range::Line(Path {}, l - 1))),
            },
            _ => Ok(front::Locator::Range(Range::File(Path {}))),
        },
        None => Err(Error::Other("Unspecified location".to_owned())),
    }
}

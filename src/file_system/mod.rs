use crate::front;
use crate::front::data::{Position, Range};
use crate::parse::ast;
use std::fmt;
use std::io;

pub use self::physical::PhysicalFs;
#[cfg(test)]
pub use self::test::MockFs;

mod physical;

pub trait FileSystem {
    fn with_file<F, T>(&self, path: Path, f: F) -> Result<T, Error>
    where
        F: FnOnce(&File) -> T;

    fn find(&self, pat: SearchPattern) -> Result<Vec<Path>, Error>;
    fn resolve_location(&self, loc: ast::Location) -> Result<front::Locator, Error>;
}

#[derive(Clone)]
pub struct File {
    pub path: Path,
    pub lines: Vec<String>,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct Path {
    key: u64,
}

#[derive(Clone, Eq, PartialEq, Debug)]
pub enum SearchPattern {
    Name(String),
}

impl From<String> for SearchPattern {
    fn from(name: String) -> SearchPattern {
        SearchPattern::Name(name)
    }
}

#[derive(Debug)]
pub enum Error {
    BadLocation(String),
    InternalError(String),
    IoError(io::Error),
    Other(String),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::BadLocation(s) => write!(f, "Invalid location: {}", s),
            Error::InternalError(s) => write!(f, "Internal error: {}", s),
            Error::IoError(e) => e.fmt(f),
            Error::Other(s) => write!(f, "File error: {}", s),
        }
    }
}

impl From<io::Error> for Error {
    fn from(e: io::Error) -> Error {
        Error::IoError(e)
    }
}

// Helper function which should only be used by file systems
fn resolve_location<Fs: FileSystem>(loc: ast::Location, fs: &Fs) -> Result<front::Locator, Error> {
    match loc.file {
        Some(f) => {
            let mut paths = fs.find(f.clone().into())?;
            if paths.is_empty() {
                return Err(Error::BadLocation(format!("no files match `{}`", f)));
            }
            if paths.len() > 1 {
                if loc.line.is_some() || loc.column.is_some() {
                    return Err(Error::BadLocation(format!(
                        "line or column specified for multiple a multi-file range"
                    )));
                }
                return Ok(front::Locator::Range(Range::MultiFile(paths)));
            }
            let path = paths.pop().unwrap();
            match loc.line {
                Some(l) if l > 0 => match loc.column {
                    Some(c) if c > 0 => Ok(front::Locator::Position(Position {
                        file: path,
                        line: l - 1,
                        column: c - 1,
                    })),
                    _ => Ok(front::Locator::Range(Range::Line(path, l - 1))),
                },
                _ => Ok(front::Locator::Range(Range::File(path))),
            }
        }
        None => Err(Error::BadLocation("unspecified location".to_owned())),
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::parse::ast::builder;

    pub struct MockFs;

    impl FileSystem for MockFs {
        fn with_file<F, T>(&self, path: Path, f: F) -> Result<T, Error>
        where
            F: FnOnce(&File) -> T,
        {
            Err(Error::Other(format!("{}", path.key)))
        }

        fn find(&self, pat: SearchPattern) -> Result<Vec<Path>, Error> {
            match &pat {
                SearchPattern::Name(s) if s == "foo.rs" => Ok(vec![Path { key: 1 }]),
                SearchPattern::Name(s) if s == "bar.rs" => Ok(vec![Path { key: 2 }]),
                SearchPattern::Name(s) if s == "baz.rs" => Ok(vec![Path { key: 3 }]),
                p => Err(Error::Other(format!("{:?}", p))),
            }
        }

        fn resolve_location(&self, loc: ast::Location) -> Result<front::Locator, Error> {
            resolve_location(loc, self)
        }
    }

    fn file_range(key: u64) -> front::Locator {
        front::Locator::Range(Range::File(Path { key }))
    }

    fn line_range(key: u64, line: usize) -> front::Locator {
        front::Locator::Range(Range::Line(Path { key }, line))
    }

    fn position(key: u64, line: usize, column: usize) -> front::Locator {
        front::Locator::Position(Position {
            file: Path { key },
            line,
            column,
        })
    }

    #[test]
    fn test_resolve_loc() {
        assert!(resolve_location(builder::location(None, None, None), &MockFs).is_err());
        assert_eq!(
            resolve_location(
                builder::location(Some("bar.rs".to_owned()), None, None),
                &MockFs
            )
            .unwrap(),
            file_range(2)
        );
        assert_eq!(
            resolve_location(
                builder::location(Some("baz.rs".to_owned()), Some(4), None),
                &MockFs
            )
            .unwrap(),
            line_range(3, 3)
        );
        assert_eq!(
            resolve_location(
                builder::location(Some("foo.rs".to_owned()), Some(4), Some(42)),
                &MockFs
            )
            .unwrap(),
            position(1, 3, 41)
        );
    }
}

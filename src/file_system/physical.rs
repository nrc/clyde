use crate::file_system::{self, File, FileSystem, Path, SearchPattern};
use crate::front;
use crate::parse::ast;
use std::path::{Path as StdPath, PathBuf};

pub struct PhysicalFs {
    root: PathBuf,
}

impl PhysicalFs {
    pub fn new(root: &StdPath) -> PhysicalFs {
        PhysicalFs {
            root: root.to_owned(),
        }
    }
}

impl FileSystem for PhysicalFs {
    fn with_file<F, T>(&self, path: &Path, f: F) -> T
    where
        F: FnOnce(&File) -> T,
    {
        unimplemented!();
    }

    fn find(&self, pat: SearchPattern) -> Result<Vec<Path>, file_system::Error> {
        unimplemented!();
    }

    fn resolve_location(&self, loc: ast::Location) -> Result<front::Locator, file_system::Error> {
        // TODO pre-cache the file?
        file_system::resolve_location(loc, self)
    }
}

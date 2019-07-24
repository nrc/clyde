use crate::ast;
use crate::file_system::{self, File, FileSystem, Path, SearchPattern};
use crate::front;
use std::cell::RefCell;
use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::fs::File as StdFile;
use std::hash::{Hash, Hasher};
use std::io::{BufRead, BufReader, Write};
use std::path::{Path as StdPath, PathBuf};

pub struct PhysicalFs {
    root: PathBuf,
    path_map: RefCell<HashMap<u64, PathBuf>>,
    file_cache: RefCell<HashMap<u64, File>>,
}

impl PhysicalFs {
    pub fn new(root: &StdPath) -> PhysicalFs {
        PhysicalFs {
            root: root.to_owned(),
            path_map: RefCell::new(HashMap::new()),
            file_cache: RefCell::new(HashMap::new()),
        }
    }

    fn insert_path(&self, path: PathBuf) -> Result<Path, file_system::Error> {
        if path.is_absolute() {
            return Err(file_system::Error::BadLocation(format!(
                "absolute path: `{}`",
                path.display()
            )));
        }

        let mut abs_path = self.root.clone();
        abs_path.push(path);
        let abs_path = abs_path.canonicalize()?;

        let mut hasher = DefaultHasher::new();
        abs_path.hash(&mut hasher);
        let key = hasher.finish();
        let mut path_map = self.path_map.borrow_mut();
        path_map.insert(key, abs_path);
        Ok(Path { key })
    }

    fn ensure_path(&self, path: Path) -> Result<(), file_system::Error> {
        {
            let file_cache = self.file_cache.borrow();
            if file_cache.contains_key(&path.key) {
                return Ok(());
            }
        }

        let file = {
            let path_map = self.path_map.borrow();
            let std_path = match path_map.get(&path.key) {
                Some(p) => p,
                None => {
                    return Err(file_system::Error::InternalError(
                        "path missing from path_map".to_owned(),
                    ))
                }
            };
            StdFile::open(std_path)?
        };
        let reader = BufReader::new(file);
        let file = File {
            path,
            lines: reader.lines().collect::<Result<Vec<_>, _>>()?,
        };

        let mut file_cache = self.file_cache.borrow_mut();
        file_cache.insert(path.key, file);
        Ok(())
    }
}

impl FileSystem for PhysicalFs {
    fn with_file<F, T>(&self, path: Path, f: F) -> Result<T, file_system::Error>
    where
        F: FnOnce(&File) -> T,
    {
        self.ensure_path(path)?;
        let file_cache = self.file_cache.borrow();
        let result = f(&file_cache[&path.key]);
        Ok(result)
    }

    fn find(&self, pat: SearchPattern) -> Result<Vec<Path>, file_system::Error> {
        // FIXME pat might be a plain name, but still be a directory and thus give a MultiFile result.
        match pat {
            SearchPattern::Name(name) => {
                let path = self.insert_path(name.into())?;
                Ok(vec![path])
            }
        }
    }

    fn resolve_location(&self, loc: ast::Location) -> Result<front::Locator, file_system::Error> {
        // FIXME pre-cache the file?
        file_system::resolve_location(loc, self)
    }

    fn show_path(&self, path: Path, w: &mut dyn Write) -> Result<(), file_system::Error> {
        // TODO unwraps should return errors
        let path_map = self.path_map.borrow();
        let path = path_map.get(&path.key).unwrap();
        let path = path.strip_prefix(&self.root).unwrap();
        write!(w, "{}", path.display()).map_err(Into::into)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::fs;
    use std::time::SystemTime;

    struct TestEnv {
        root: PathBuf,
    }

    impl TestEnv {
        fn init() -> TestEnv {
            // We use a funky directory since many tests might create these mini-envs
            // and be executed in parallel.
            let root = PathBuf::from(format!(
                "./target/test-{}",
                SystemTime::now()
                    .duration_since(SystemTime::UNIX_EPOCH)
                    .unwrap()
                    .as_nanos()
            ));
            fs::create_dir_all(&root).unwrap();
            let env = TestEnv { root };
            env.create_file("foo.rs");
            env.create_file("bar.rs");

            env
        }

        fn create_file(&self, name: &str) {
            let mut f = fs::File::create(&self.path(name)).unwrap();
            for i in 0..100 {
                writeln!(f, "line {} of {}", i, name).unwrap();
            }
        }

        fn fs(&self) -> PhysicalFs {
            PhysicalFs::new(&self.root)
        }

        fn path(&self, s: &str) -> PathBuf {
            let mut path = self.root.clone();
            path.push(s);
            path
        }
    }

    impl Drop for TestEnv {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.root);
        }
    }

    #[test]
    fn test_find() {
        let env = TestEnv::init();
        let fs = env.fs();
        let results = fs.find("foo.rs".to_owned().into()).unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(
            fs.path_map.borrow().get(&results[0].key).unwrap(),
            &env.path("foo.rs").canonicalize().unwrap()
        );
    }

    #[test]
    fn test_with_file() {
        let env = TestEnv::init();
        let fs = env.fs();
        let path = fs.find("foo.rs".to_owned().into()).unwrap().pop().unwrap();
        fs.with_file(path, |file| {
            assert_eq!(file.path.key, path.key);
            assert_eq!(file.lines.len(), 100);
            assert_eq!(file.lines[32], "line 32 of foo.rs");
        })
        .unwrap();
    }
}

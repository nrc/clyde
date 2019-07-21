use crate::file_system::FileSystem;
use crate::front;
use crate::parse::{self, ast};

pub(crate) mod repl;

pub trait Environment {
    type ParseContext: parse::EnvContext;
    type Fs: FileSystem;

    fn exec_meta(&self, mk: ast::MetaKind) -> Result<(), front::Error>;
    fn show(&self, s: &str) -> Result<(), front::Error>;
    fn lookup_var(&self, var: &front::MetaVar) -> Result<front::Value, front::Error>;
    fn lookup_numeric_var(&self, id: isize) -> Result<front::Value, front::Error>;
    fn file_system(&self) -> &Self::Fs;
}

#[cfg(test)]
pub mod mock {
    use super::*;
    use crate::file_system::PhysicalFs;

    pub struct MockEnv;

    impl Environment for MockEnv {
        type ParseContext = ();
        // TODO MockFs
        type Fs = PhysicalFs;

        fn exec_meta(&self, mk: ast::MetaKind) -> Result<(), front::Error> {
            Err(front::Error::Other(match mk {
                ast::MetaKind::Help => "help".to_owned(),
                ast::MetaKind::Exit => "exit".to_owned(),
            }))
        }

        fn show(&self, s: &str) -> Result<(), front::Error> {
            eprintln!("show: {}", s);
            Err(front::Error::Other(s.to_owned()))
        }

        fn lookup_var(&self, _: &front::MetaVar) -> Result<front::Value, front::Error> {
            Err(front::Error::Other(
                "MockEnv does not support var lookup".to_owned(),
            ))
        }

        fn lookup_numeric_var(&self, _: isize) -> Result<front::Value, front::Error> {
            Err(front::Error::Other(
                "MockEnv does not support numeric var lookup".to_owned(),
            ))
        }

        fn file_system(&self) -> &Self::Fs {
            unimplemented!();
        }
    }

    impl parse::EnvContext for () {
        fn clone(&self) -> Box<dyn parse::EnvContext> {
            Box::new(())
        }
    }
}

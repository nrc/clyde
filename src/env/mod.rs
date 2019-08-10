use crate::back::Backend;
use crate::file_system::FileSystem;
use crate::front::{self, Show};
use crate::parse::{self, ast};
use std::rc::Rc;

pub(crate) mod repl;

pub trait Environment {
    type ParseContext: parse::EnvContext;
    type Fs: FileSystem;

    fn exec_meta(&self, mk: ast::MetaKind) -> Result<(), front::Error>;
    fn show(&self, s: &impl Show) -> Result<(), front::Error>;
    fn lookup_var(&self, var: &front::MetaVar) -> Result<front::Value, front::Error>;
    fn lookup_numeric_var(&self, id: isize) -> Result<front::Value, front::Error>;
    fn file_system(&self) -> &Self::Fs;
    fn backend(&self) -> Rc<dyn Backend>;
}

#[cfg(test)]
pub mod mock {
    use super::*;
    use crate::file_system::MockFs;

    pub struct MockEnv;

    impl Environment for MockEnv {
        type ParseContext = ();
        type Fs = MockFs;

        fn exec_meta(&self, mk: ast::MetaKind) -> Result<(), front::Error> {
            Err(front::Error::Other(match mk {
                ast::MetaKind::Help => "help".to_owned(),
                ast::MetaKind::Exit => "exit".to_owned(),
            }))
        }

        fn show(&self, s: &impl Show) -> Result<(), front::Error> {
            Err(front::Error::Other(s.show_str(self)))
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
            &MockFs
        }

        fn backend(&self) -> Rc<dyn Backend> {
            unimplemented!()
        }
    }

    impl parse::EnvContext for () {
        fn clone(&self) -> Box<dyn parse::EnvContext> {
            Box::new(())
        }
    }
}

pub use self::data::{Locator, MetaVar, Value};
use crate::env::Environment;
use crate::file_system::{self, FileSystem};
use crate::ast;
use std::collections::HashMap;
use std::fmt;
use std::io::{self, Write};

pub mod data;

pub struct Interpreter<'a, Env: Environment> {
    env: &'a Env,
    symbols: SymbolTable,
}

impl<'a, Env: Environment> Interpreter<'a, Env> {
    pub fn new(env: &'a Env) -> Interpreter<'a, Env> {
        Interpreter {
            env,
            symbols: SymbolTable::default(),
        }
    }

    pub fn interpret(mut self, program: ast::Program) -> Result<SymbolTable, Error> {
        for stmt in program.stmts {
            self.interpret_stmt(stmt)?;
        }

        Ok(self.symbols)
    }

    fn interpret_stmt(&mut self, stmt: ast::Statement) -> Result<(), Error> {
        match stmt.kind {
            ast::StatementKind::Expr(expr) => {
                let value = self.interpret_expr(expr)?;
                self.env.show(&value)
            }
            ast::StatementKind::Show(sh) => {
                let value = self.interpret_expr(sh.expr.kind)?;
                self.env.show(&value)
            }
            ast::StatementKind::Meta(mk) => self.env.exec_meta(mk),
        }
    }

    fn interpret_expr(&mut self, expr: ast::ExprKind) -> Result<Value, Error> {
        match expr {
            ast::ExprKind::Void => Ok(Value::void()),
            ast::ExprKind::MetaVar(kind) => self.lookup_var(kind),
            ast::ExprKind::Location(loc) => {
                let loc = self.env.file_system().resolve_location(loc)?;
                Ok(loc.into())
            }
            _ => unimplemented!(),
        }
    }

    fn lookup_var(&mut self, kind: ast::MetaVarKind) -> Result<Value, Error> {
        match kind {
            ast::MetaVarKind::Dollar => self.env.lookup_numeric_var(-1),
            ast::MetaVarKind::Numeric(n) => self.env.lookup_numeric_var(n as isize),
            ast::MetaVarKind::Named(id) => {
                let var = MetaVar { name: id.name };
                match self.symbols.lookup(&var) {
                    Some(v) => Ok(v),
                    None => {
                        let value = self.env.lookup_var(&var)?;
                        self.symbols.variables.insert(var, value.clone());
                        Ok(value)
                    }
                }
            }
        }
    }
}

pub struct SymbolTable {
    variables: HashMap<MetaVar, Value>,
    result: Value,
}

impl SymbolTable {
    fn lookup(&self, var: &MetaVar) -> Option<Value> {
        self.variables.get(var).map(Clone::clone)
    }
}

impl Default for SymbolTable {
    fn default() -> SymbolTable {
        SymbolTable {
            variables: HashMap::new(),
            result: Value::void(),
        }
    }
}

pub trait Show {
    fn show(&self, w: &mut dyn Write, env: &impl Environment) -> io::Result<()>;
    fn to_string(&self, env: &impl Environment) -> String {
        let mut buf: Vec<u8> = Vec::new();
        self.show(&mut buf, env).unwrap();
        String::from_utf8(buf).unwrap()
    }
}

impl<T: fmt::Display> Show for T {
    fn show(&self, w: &mut dyn Write, _: &impl Environment) -> io::Result<()> {
        write!(w, "{}", self).into()
    }
}

#[derive(Debug, Clone)]
pub enum Error {
    VarNotFound(MetaVar),
    Other(String),
}

impl From<file_system::Error> for Error {
    fn from(e: file_system::Error) -> Error {
        Error::Other(fmt::Display::to_string(&e))
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::VarNotFound(v) => write!(f, "Variable not found: `{}`", v),
            Error::Other(s) => write!(f, "{}", s),
        }
    }
}

#[cfg(test)]
mod test {
    use super::data::ValueKind;
    use super::*;
    use crate::env::mock::MockEnv;
    use crate::ast::builder;

    fn assert_err<T: fmt::Debug>(e: Result<T, Error>, s: &str) {
        if let Err(Error::Other(msg)) = &e {
            if msg == s {
                return;
            }
        }

        panic!("Expected `{}`, found {:?}", s, e);
    }

    #[test]
    fn test_void() {
        let mut interp = Interpreter::new(&MockEnv);
        if let ValueKind::Void = interp.interpret_expr(ast::ExprKind::Void).unwrap().kind {
            return;
        }
        panic!();
    }

    #[test]
    fn test_var_lookup() {
        let mut interp = Interpreter::new(&MockEnv);
        assert!(interp
            .lookup_var(ast::MetaVarKind::Named(builder::ident("foo")))
            .is_err());

        interp
            .symbols
            .variables
            .insert(MetaVar::new("foo"), Value::void());
        assert!(interp
            .lookup_var(ast::MetaVarKind::Named(builder::ident("foo")))
            .is_ok());
    }

    #[test]
    fn test_meta() {
        let mut interp = Interpreter::new(&MockEnv);
        assert_err(
            interp.interpret_stmt(builder::meta_stmt(ast::MetaKind::Exit)),
            "exit",
        );
        assert_err(
            interp.interpret_stmt(builder::meta_stmt(ast::MetaKind::Help)),
            "help",
        );
    }

    #[test]
    fn test_show() {
        let mut interp = Interpreter::new(&MockEnv);
        assert_err(interp.interpret_stmt(builder::show(builder::void())), "()");
    }

    // TODO test locations
    #[test]
    fn test_location() {}
}

pub use self::data::{Locator, MetaVar, Type, Value};
use self::function::Function;
use crate::ast;
use crate::back;
use crate::env::Environment;
use crate::file_system::{self, FileSystem};
use std::collections::HashMap;
use std::fmt;
use std::io::{self, Write};

pub mod data;
mod function;
mod query;

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
                self.show_result(&value)
            }
            ast::StatementKind::Meta(mk) => self.env.exec_meta(mk),
            ast::StatementKind::ApplyShorthand(a) => {
                let value = self.interpret_apply(a)?;
                self.show_result(&value)
            } //_ => unimplemented!(),
        }
    }

    fn show_result(&self, value: &Value) -> Result<(), Error> {
        if !value.kind.is_void() {
            self.env.show(value)?;
        }
        Ok(())
    }

    fn interpret_expr(&mut self, expr: ast::ExprKind) -> Result<Value, Error> {
        match expr {
            ast::ExprKind::Void => Ok(Value::void()),
            ast::ExprKind::MetaVar(kind) => self.lookup_var(&kind),
            ast::ExprKind::Location(loc) => {
                let loc = self.env.file_system().resolve_location(loc)?;
                Ok(loc.into())
            }
            ast::ExprKind::Apply(a) => self.interpret_apply(a),
            ast::ExprKind::Field(p) => self.interpret_apply(p.into()),
            _ => unimplemented!(),
        }
    }

    fn type_expr(&mut self, expr: &ast::ExprKind) -> Result<Type, Error> {
        match expr {
            ast::ExprKind::Void => Ok(Type::Void),
            ast::ExprKind::MetaVar(kind) => self.lookup_var(kind).map(|val| val.ty),
            ast::ExprKind::Location(_) => Ok(Type::Location),
            ast::ExprKind::Apply(a) => self.type_apply(a),
            ast::ExprKind::Field(p) => self.type_apply(&(*p).clone().into()),
            _ => unimplemented!(),
        }
    }

    fn interpret_apply(&mut self, apply: ast::Apply) -> Result<Value, Error> {
        macro_rules! interpret {
            ($e: expr, $($fn: ident),*) => {
                match &*$e {
                    $(function::$fn::NAME => {
                        let fun = function::$fn {};
                        function::$fn::ARITY.check(&apply.args)?;
                        fun.ty(self, &apply.lhs, &apply.args)?;
                        fun.eval(self, apply.lhs, apply.args)
                    })*
                    _ => Err(Error::UnknownFunction($e))
                }
            }
        };

        interpret!(apply.ident.name, Select, Show, Idents)
    }

    fn type_apply(&mut self, apply: &ast::Apply) -> Result<Type, Error> {
        macro_rules! typ {
            ($e: expr, $($fn: ident),*) => {
                match &*$e {
                    $(function::$fn::NAME => {
                        let fun = function::$fn {};
                        function::$fn::ARITY.check(&apply.args)?;
                        fun.ty(self, &apply.lhs, &apply.args)
                    })*
                    _ => Err(Error::UnknownFunction($e.to_owned()))
                }
            }
        };

        typ!(apply.ident.name, Select, Show, Idents)
    }

    fn lookup_var(&mut self, kind: &ast::MetaVarKind) -> Result<Value, Error> {
        match kind {
            ast::MetaVarKind::Dollar => self.env.lookup_numeric_var(-1),
            ast::MetaVarKind::Numeric(n) => self.env.lookup_numeric_var(*n as isize),
            ast::MetaVarKind::Named(id) => {
                let var = MetaVar {
                    name: id.name.clone(),
                };
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
    fn show(&self, w: &mut dyn Write, env: &impl Environment) -> Result<(), Error>;
    fn show_str(&self, env: &impl Environment) -> String {
        let mut buf: Vec<u8> = Vec::new();
        self.show(&mut buf, env).unwrap();
        String::from_utf8(buf).unwrap()
    }
}

impl<T: fmt::Display> Show for T {
    fn show(&self, w: &mut dyn Write, _: &impl Environment) -> Result<(), Error> {
        write!(w, "{}", self).map_err(Into::into)
    }
}

#[derive(Debug)]
pub enum Error {
    IoError(io::Error),
    VarNotFound(MetaVar),
    UnknownFunction(String),
    TypeError(String),
    Other(String),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::IoError(e) => e.fmt(f),
            Error::VarNotFound(v) => write!(f, "Variable not found: `{}`", v),
            Error::UnknownFunction(s) => write!(f, "Unknown function: `{}`", s),
            Error::TypeError(s) => write!(f, "{}", s),
            Error::Other(s) => write!(f, "{}", s),
        }
    }
}

impl From<file_system::Error> for Error {
    fn from(e: file_system::Error) -> Error {
        Error::Other(e.to_string())
    }
}

impl From<back::Error> for Error {
    fn from(e: back::Error) -> Error {
        Error::Other(e.to_string())
    }
}

impl From<io::Error> for Error {
    fn from(e: io::Error) -> Error {
        Error::IoError(e)
    }
}

#[cfg(test)]
mod test {
    use super::data::ValueKind;
    use super::*;
    use crate::ast::builder;
    use crate::env::mock::MockEnv;

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
        // FIXME not implemented yet
        // assert_err(interp.interpret_stmt(builder::show(builder::void())), "()");
    }
}

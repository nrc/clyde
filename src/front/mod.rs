use self::data::{Position, Range};
use crate::env::Environment;
use crate::file_system::{self, FileSystem};
use crate::parse::ast;
use std::collections::HashMap;
use std::fmt;

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
                self.interpret_expr(expr)?;
                Ok(())
            }
            ast::StatementKind::Show(sh) => {
                let value = self.interpret_expr(sh.expr.kind)?;
                self.env.show(&value.to_string())
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

#[derive(Clone, Eq, PartialEq, Hash, Debug)]
pub struct MetaVar {
    name: String,
}

impl MetaVar {
    fn new(name: &str) -> MetaVar {
        MetaVar {
            name: name.to_owned(),
        }
    }
}

impl fmt::Display for MetaVar {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.name.fmt(f)
    }
}

#[derive(Clone, Eq, PartialEq, Debug)]
pub struct Value {
    ty: Type,
    kind: ValueKind,
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.kind.fmt(f)
    }
}

impl Value {
    fn void() -> Value {
        Value {
            ty: Type::Void,
            kind: ValueKind::Void,
        }
    }

    fn number(n: usize) -> Value {
        Value {
            ty: Type::Number,
            kind: ValueKind::Number(n),
        }
    }
}

#[derive(Clone, Eq, PartialEq, Debug)]
pub enum Type {
    Void,
    Number,
    Set(Box<Type>),
    Position,
    Range,
}

#[derive(Clone, Eq, PartialEq, Debug)]
pub enum ValueKind {
    Void,
    Number(usize),
    Set(Vec<Value>),
    Position(Position),
    Range(Range),
}

impl fmt::Display for ValueKind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ValueKind::Void => write!(f, "()"),
            ValueKind::Number(n) => write!(f, "{}", n),
            ValueKind::Set(v) => {
                if v.len() < 5 {
                    write!(f, "[")?;
                    let mut first = true;
                    for v in v {
                        write!(f, "{}", v)?;
                        if first {
                            first = false;
                        } else {
                            write!(f, ", ")?;
                        }
                    }
                    write!(f, "]")
                } else {
                    write!(f, "[...]*{}", v.len())
                }
            }
            ValueKind::Position(_) => write!(f, "TODO"),
            ValueKind::Range(_) => write!(f, "TODO"),
        }
    }
}

#[derive(Clone, Eq, PartialEq, Debug)]
pub enum Locator {
    Position(Position),
    Range(Range),
}

impl From<Locator> for Value {
    fn from(loc: Locator) -> Value {
        match loc {
            Locator::Position(p) => Value {
                ty: Type::Position,
                kind: ValueKind::Position(p),
            },
            Locator::Range(r) => Value {
                ty: Type::Range,
                kind: ValueKind::Range(r),
            },
        }
    }
}

#[derive(Debug, Clone)]
pub enum Error {
    VarNotFound(MetaVar),
    Other(String),
}

impl From<file_system::Error> for Error {
    fn from(e: file_system::Error) -> Error {
        Error::Other(e.to_string())
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::env::mock::MockEnv;
    use crate::parse::ast::builder;

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
        assert_eq!(
            interp.interpret_expr(ast::ExprKind::Void).unwrap(),
            Value::void()
        );
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

    #[test]
    fn test_value_display() {
        assert_eq!(Value::void().to_string(), "()");

        // TODO value.display
    }
}

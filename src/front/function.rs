use crate::ast;
use crate::env::Environment;
use crate::front::data::{Type, Value, ValueKind};
use crate::front::{Error, Interpreter};
use std::fmt;

pub enum Arity {
    None,
    Exactly(usize),
    AtLeast(usize),
}

impl Arity {
    pub fn check(&self, args: &[ast::Expr]) -> Result<(), Error> {
        match (self, args.len()) {
            (Arity::None, 0) => Ok(()),
            (Arity::Exactly(n), l) if l == *n => Ok(()),
            (Arity::AtLeast(n), l) if l >= *n => Ok(()),
            (_, l) => Err(Error::TypeError(format!(
                "Incorrect arguments, expected: {}, found {}",
                self, l
            ))),
        }
    }
}

impl fmt::Display for Arity {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Arity::None => write!(f, "0"),
            Arity::Exactly(n) => n.fmt(f),
            Arity::AtLeast(n) => write!(f, "{} or more", n),
        }
    }
}

pub trait Function {
    const NAME: &'static str;
    const ARITY: Arity;

    fn eval(
        &self,
        interpreter: &mut Interpreter<'_, impl Environment>,
        lhs: Box<ast::Expr>,
        args: Vec<ast::Expr>,
    ) -> Result<Value, Error>;

    fn ty(
        &self,
        interpreter: &mut Interpreter<'_, impl Environment>,
        lhs: &ast::Expr,
        args: &[ast::Expr],
    ) -> Result<Type, Error>;
}

pub struct Show {}

impl Function for Show {
    const NAME: &'static str = "show";
    const ARITY: Arity = Arity::None;

    fn eval(
        &self,
        interpreter: &mut Interpreter<'_, impl Environment>,
        lhs: Box<ast::Expr>,
        _: Vec<ast::Expr>,
    ) -> Result<Value, Error> {
        let lhs = interpreter.interpret_expr(lhs.kind)?;
        if lhs.ty.is_query() {
            // TODO
        } else {
            interpreter.env.show(&lhs)?;
        }
        Ok(Value::void())
    }

    fn ty(
        &self,
        _: &mut Interpreter<'_, impl Environment>,
        _: &ast::Expr,
        _: &[ast::Expr],
    ) -> Result<Type, Error> {
        Ok(Type::Void)
    }
}

pub struct Select {}

impl Function for Select {
    const NAME: &'static str = "select";
    const ARITY: Arity = Arity::None;

    fn eval(
        &self,
        interpreter: &mut Interpreter<'_, impl Environment>,
        lhs: Box<ast::Expr>,
        _: Vec<ast::Expr>,
    ) -> Result<Value, Error> {
        let lhs = interpreter.interpret_expr(lhs.kind)?;
        match &lhs.kind {
            ValueKind::Query(q) => unimplemented!(),
            _ => Err(Error::TypeError(format!(
                "Expected query, found {:?}",
                lhs.ty
            ))),
        }
    }

    fn ty(
        &self,
        interpreter: &mut Interpreter<'_, impl Environment>,
        lhs: &ast::Expr,
        _: &[ast::Expr],
    ) -> Result<Type, Error> {
        match interpreter.type_expr(&lhs.kind)? {
            Type::Query(ty) => Ok(*ty),
            ty => Err(Error::TypeError(format!("Expected query, found {:?}", ty))),
        }
    }
}

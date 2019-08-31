use crate::ast;
use crate::env::Environment;
use crate::front::data::{Type, Value, ValueKind};
use crate::front::{query, Error, Interpreter};
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
            let value = lhs.expect_query().eval(&*interpreter.env.backend())?;
            interpreter.env.show(&value)?;
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
            ValueKind::Query(q) => q.eval(&*interpreter.env.backend()),
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

pub struct Pick {}

impl Function for Pick {
    const NAME: &'static str = "pick";
    const ARITY: Arity = Arity::None;

    fn eval(
        &self,
        interpreter: &mut Interpreter<'_, impl Environment>,
        lhs: Box<ast::Expr>,
        _: Vec<ast::Expr>,
    ) -> Result<Value, Error> {
        let lhs = interpreter.interpret_expr(lhs.kind)?;
        match &lhs.kind {
            ValueKind::Query(_) => {
                let ty = lhs.ty.unquery().expect_set_inner();
                Ok(Value {
                    kind: ValueKind::Query(query::Pick::new(lhs.into(), ty.clone())),
                    ty: Type::Query(Box::new(ty)),
                })
            }
            ValueKind::Set(vs) if vs.is_empty() => Err(Error::EmptySet),
            ValueKind::Set(vs) => Ok(vs[0].clone()),
            _ => Err(Error::TypeError(format!(
                "Expected set, found {:?}",
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
        let lhs_ty = interpreter.type_expr(&lhs.kind)?;
        let inner = match lhs_ty.unquery() {
            Type::Set(ty) => *ty,
            _ => {
                return Err(Error::TypeError(format!(
                    "Expected set, found {:?}",
                    lhs_ty
                )))
            }
        };

        if lhs_ty.is_query() {
            Ok(Type::Query(Box::new(inner)))
        } else {
            Ok(inner)
        }
    }
}

pub struct Idents {}

impl Function for Idents {
    const NAME: &'static str = "idents";
    const ARITY: Arity = Arity::None;

    fn eval(
        &self,
        interpreter: &mut Interpreter<'_, impl Environment>,
        lhs: Box<ast::Expr>,
        _: Vec<ast::Expr>,
    ) -> Result<Value, Error> {
        let lhs = interpreter.interpret_expr(lhs.kind)?;
        Ok(Value {
            kind: ValueKind::Query(query::Idents::new(lhs.into())),
            ty: Type::Query(Box::new(Type::Set(Box::new(Type::Identifier)))),
        })
    }

    fn ty(
        &self,
        interpreter: &mut Interpreter<'_, impl Environment>,
        lhs: &ast::Expr,
        _: &[ast::Expr],
    ) -> Result<Type, Error> {
        let ty_lhs = interpreter.type_expr(&lhs.kind)?;
        if !ty_lhs.is_location() {
            return Err(Error::TypeError(format!(
                "Expected location, found {:?}",
                ty_lhs
            )));
        }

        Ok(Type::Query(Box::new(Type::Set(Box::new(Type::Identifier)))))
    }
}

pub struct Definition {}

impl Function for Definition {
    const NAME: &'static str = "def";
    const ARITY: Arity = Arity::None;

    fn eval(
        &self,
        interpreter: &mut Interpreter<'_, impl Environment>,
        lhs: Box<ast::Expr>,
        _: Vec<ast::Expr>,
    ) -> Result<Value, Error> {
        let lhs = interpreter.interpret_expr(lhs.kind)?;
        Ok(Value {
            kind: ValueKind::Query(query::Definition::new(lhs.clone().into(), lhs.ty.clone())),
            ty: lhs.ty,
        })
    }

    fn ty(
        &self,
        interpreter: &mut Interpreter<'_, impl Environment>,
        lhs: &ast::Expr,
        _: &[ast::Expr],
    ) -> Result<Type, Error> {
        let ty_lhs = interpreter.type_expr(&lhs.kind)?;
        match ty_lhs.unquery() {
            Type::Identifier => Ok(Type::Query(Box::new(Type::Definition))),
            Type::Set(ref inner) if &**inner == &Type::Identifier => {
                Ok(Type::Query(Box::new(Type::Set(Box::new(Type::Definition)))))
            }
            _ => Err(Error::TypeError(format!(
                "Expected identifier, found {:?}",
                ty_lhs
            ))),
        }
    }
}

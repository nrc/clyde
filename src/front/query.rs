use crate::back::Backend;
use crate::front::data::{Type, Value, ValueKind};
use crate::front::Error;

#[derive(Clone)]
pub enum Query {
    Ready(Box<Value>),
    Function(Fun),
}

impl Query {
    pub fn ready(value: Value) -> Query {
        Query::Ready(Box::new(value))
    }

    pub fn eval(&self, back: &dyn Backend) -> Result<Value, Error> {
        match self {
            Query::Ready(v) => Ok((**v).clone()),
            Query::Function(f) => f.def.eval(f, back),
        }
    }
}

#[derive(Clone)]
pub struct Fun {
    pub def: &'static dyn Function,
    pub ty: Type,
    pub lhs: Box<Query>,
    pub args: Vec<Value>,
}

pub trait Function {
    fn eval(&self, f: &Fun, back: &dyn Backend) -> Result<Value, Error>;
}

#[derive(Clone)]
pub struct Pick;

impl Pick {
    pub fn new(lhs: Query, ty: Type) -> Query {
        Query::Function(Fun {
            def: &Pick,
            ty,
            lhs: Box::new(lhs),
            args: vec![],
        })
    }
}

impl Function for Pick {
    fn eval(&self, f: &Fun, back: &dyn Backend) -> Result<Value, Error> {
        let lhs = f.lhs.eval(back)?;
        match lhs.kind {
            ValueKind::Set(s) => Ok(s[0].clone()),
            _ => {
                return Err(Error::TypeError(format!(
                    "Unexpected runtime type, expected: set, found: {:?}",
                    lhs.ty
                )))
            }
        }
    }
}

#[derive(Clone)]
pub struct Idents;

impl Idents {
    pub fn new(lhs: Query) -> Query {
        Query::Function(Fun {
            def: &Idents,
            ty: Type::Set(Box::new(Type::Identifier)),
            lhs: Box::new(lhs),
            args: vec![],
        })
    }
}

impl Function for Idents {
    fn eval(&self, f: &Fun, back: &dyn Backend) -> Result<Value, Error> {
        let lhs = f.lhs.eval(back)?;
        let idents = match lhs.kind {
            ValueKind::Position(p) => back.ident_at(p.clone())?.into_iter().collect(),
            ValueKind::Range(r) => back.idents_in(r.clone())?,
            ValueKind::Set(_) => unimplemented!(),
            _ => {
                return Err(Error::TypeError(format!(
                    "Unexpected runtime type, expected: location, found: {:?}",
                    lhs.ty
                )))
            }
        };

        Ok(Value {
            kind: ValueKind::Set(
                idents
                    .into_iter()
                    .map(|i| Value {
                        kind: ValueKind::Identifier(i),
                        ty: Type::Identifier,
                    })
                    .collect(),
            ),
            ty: f.ty.clone(),
        })
    }
}

#[derive(Clone)]
pub struct Definition;

impl Definition {
    pub fn new(lhs: Query, ty: Type) -> Query {
        Query::Function(Fun {
            def: &Definition,
            ty,
            lhs: Box::new(lhs),
            args: vec![],
        })
    }
}

impl Function for Definition {
    fn eval(&self, f: &Fun, back: &dyn Backend) -> Result<Value, Error> {
        let lhs = f.lhs.eval(back)?;
        let def = match lhs.kind {
            ValueKind::Identifier(id) => back.definition(id.clone())?,
            ValueKind::Set(_) => unimplemented!(),
            _ => {
                return Err(Error::TypeError(format!(
                    "Unexpected runtime type, expected: identifier, found: {:?}",
                    lhs.ty
                )))
            }
        };

        Ok(Value {
            kind: ValueKind::Definition(def),
            ty: Type::Definition,
        })
    }
}

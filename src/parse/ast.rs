use super::Context;
use derive_new::new;

pub trait Node {}

pub struct Statement {
    pub kind: StatementKind,
    pub ctx: Context,
}

impl Node for Statement {}

pub enum StatementKind {
    Expr(ExprKind),
    Show(Show),
    Meta(MetaKind),
}

pub struct Expr {
    pub kind: ExprKind,
    pub ctx: Context,
}

impl Node for Expr {}

pub enum ExprKind {
    Select(Select),
    MetaVar(MetaVarKind),
    // ()
    Void,
    // (foo expr)
    Apply(Apply),
    // (:...)
    Location(Location),
}

// FIXME Select and Show could just use Apply
pub struct Select {
    pub multiplicity: Multiplicity,
    pub filters: Vec<Expr>,
    pub ctx: Context,
}

impl Node for Select {}

pub struct Show {
    pub expr: Box<Expr>,
    pub ctx: Context,
}

impl Node for Show {}

pub struct Apply {
    pub ident: Identifier,
    pub args: Vec<Expr>,
    pub ctx: Context,
}

impl Node for Apply {}

#[derive(new)]
pub struct Location {
    pub file: Option<String>,
    pub line: Option<usize>,
    pub column: Option<usize>,
    pub ctx: Context,
}

impl Node for Location {}

#[derive(Clone)]
pub enum MetaVarKind {
    Dollar,
    Numeric(usize),
    Named(Identifier),
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum MetaKind {
    Exit,
    Help,
}

#[derive(new, Clone)]
pub struct Identifier {
    pub name: String,
    pub ctx: Context,
}

impl Node for Identifier {}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum Multiplicity {
    Many,
    One,
}

use super::Context;
use derive_new::new;

pub trait Node {}

#[derive(Clone)]
pub struct Program {
    pub stmts: Vec<Statement>,
    pub ctx: Context,
}

impl Node for Program {}

#[derive(Clone)]
pub struct Statement {
    pub kind: StatementKind,
    pub ctx: Context,
}

impl Node for Statement {}

#[derive(Clone)]
pub enum StatementKind {
    Expr(ExprKind),
    // foo expr
    ApplyShorthand(Apply),
    Meta(MetaKind),
}

#[derive(Clone)]
pub struct Expr {
    pub kind: ExprKind,
    pub ctx: Context,
}

impl Node for Expr {}

#[derive(Clone)]
pub enum ExprKind {
    MetaVar(MetaVarKind),
    // ()
    Void,
    // expr->foo
    Apply(Apply),
    // (:...)
    Location(Location),
    // expr.foo
    Projection(Projection),
}

#[derive(Clone)]
pub struct Apply {
    pub ident: Identifier,
    pub lhs: Box<Expr>,
    pub args: Vec<Expr>,
    pub ctx: Context,
}

impl Node for Apply {}

#[derive(Clone)]
pub struct Projection {
    pub ident: Identifier,
    pub lhs: Box<Expr>,
    pub ctx: Context,
}

impl Node for Projection {}

impl From<Projection> for Apply {
    fn from(p: Projection) -> Apply {
        Apply {
            ident: p.ident,
            lhs: p.lhs,
            args: Vec::new(),
            ctx: p.ctx,
        }
    }
}

#[derive(new, Clone)]
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
    Numeric(isize),
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

#[cfg(test)]
pub mod builder {
    use super::*;

    pub fn ctx() -> Context {
        Context::default()
    }

    pub fn ident(s: &str) -> Identifier {
        Identifier {
            name: s.to_owned(),
            ctx: ctx(),
        }
    }

    pub fn show(e: Expr) -> Statement {
        Statement {
            kind: StatementKind::ApplyShorthand(Apply {
                ident: ident("show"),
                lhs: Box::new(e),
                args: vec![],
                ctx: ctx(),
            }),
            ctx: ctx(),
        }
    }

    pub fn void() -> Expr {
        Expr {
            kind: ExprKind::Void,
            ctx: ctx(),
        }
    }

    pub fn meta_stmt(mk: MetaKind) -> Statement {
        Statement {
            kind: StatementKind::Meta(mk),
            ctx: ctx(),
        }
    }

    pub fn location(file: Option<String>, line: Option<usize>, column: Option<usize>) -> Location {
        Location {
            file,
            line,
            column,
            ctx: ctx(),
        }
    }
}

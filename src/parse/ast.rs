use super::Context;

pub struct Statement {
    kind: StatmentKind,
    ctx: Context,
}

pub enum StatmentKind {
    Select,
    MetaVar,
    Meta,
}

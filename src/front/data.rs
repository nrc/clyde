use super::Show;
use crate::env::Environment;
use crate::file_system::Path;
use std::fmt;
use std::io::{self, Write};

#[derive(Clone, Eq, PartialEq, Hash, Debug)]
pub struct MetaVar {
    pub name: String,
}

impl MetaVar {
    pub fn new(name: &str) -> MetaVar {
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

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Value {
    pub ty: Type,
    pub kind: ValueKind,
}

impl Show for Value {
    fn show(&self, w: &mut dyn Write, env: &impl Environment) -> io::Result<()> {
        self.kind.show(w, env)
    }
}

impl Value {
    pub fn void() -> Value {
        Value {
            ty: Type::Void,
            kind: ValueKind::Void,
        }
    }

    pub fn number(n: usize) -> Value {
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

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ValueKind {
    Void,
    Number(usize),
    Set(Vec<Value>),
    Position(Position),
    Range(Range),
}

impl Show for ValueKind {
    fn show(&self, w: &mut dyn Write, env: &impl Environment) -> io::Result<()> {
        match self {
            ValueKind::Void => write!(w, "()"),
            ValueKind::Number(n) => write!(w, "{}", n),
            ValueKind::Set(v) => {
                if v.len() < 5 {
                    write!(w, "[")?;
                    let mut first = true;
                    for v in v {
                        if first {
                            first = false;
                        } else {
                            write!(w, ", ")?;
                        }
                        v.show(w, env)?;
                    }
                    write!(w, "]")
                } else {
                    write!(w, "[...]*{}", v.len())
                }
            }
            ValueKind::Position(_) => write!(w, "TODO"),
            ValueKind::Range(_) => write!(w, "TODO"),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
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

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Position {
    pub file: Path,
    pub line: usize,
    pub column: usize,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Range {
    File(Path),
    MultiFile(Vec<Path>),
    Line(Path, usize),
    Span(Span),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Span {
    pub file: Path,
    pub start_line: usize,
    pub start_column: usize,
    pub end_line: usize,
    pub end_column: usize,
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::env::mock::MockEnv;

    #[test]
    fn test_value_display() {
        assert_eq!(Value::void().to_string(&MockEnv), "()");
        assert_eq!(Value::number(42).to_string(&MockEnv), "42");
        let set = Value {
            kind: ValueKind::Set(vec![Value::number(1), Value::number(2), Value::number(3)]),
            ty: Type::Set(Box::new(Type::Number)),
        };
        assert_eq!(set.to_string(&MockEnv), "[1, 2, 3]");
        let set = Value {
            kind: ValueKind::Set(vec![
                Value::number(1),
                Value::number(2),
                Value::number(3),
                Value::number(3),
                Value::number(3),
                Value::number(3),
                Value::number(3),
                Value::number(3),
            ]),
            ty: Type::Set(Box::new(Type::Number)),
        };
        assert_eq!(set.to_string(&MockEnv), "[...]*8");
    }
}

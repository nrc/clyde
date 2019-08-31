use super::{query::Query, Error, Show};
use crate::env::Environment;
use crate::file_system::{FileSystem, Path};
use derive_new::new;
use std::fmt;
use std::io::Write;

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

#[derive(Clone)]
pub struct Value {
    pub ty: Type,
    pub kind: ValueKind,
}

impl Show for Value {
    fn show(&self, w: &mut dyn Write, env: &impl Environment) -> Result<(), Error> {
        self.kind.show(w, env)
    }
}

impl From<Value> for Query {
    fn from(value: Value) -> Query {
        match value.kind {
            ValueKind::Query(q) => q,
            _ => Query::ready(value),
        }
    }
}

impl fmt::Debug for Value {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "<Value (type: {:?})>", self.ty)
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

    pub fn string(s: String) -> Value {
        Value {
            ty: Type::String,
            kind: ValueKind::String(s),
        }
    }

    pub fn expect_query(self) -> Query {
        match self.kind {
            ValueKind::Query(q) => q,
            _ => panic!(),
        }
    }

    pub fn expect_string(self) -> String {
        match self.kind {
            ValueKind::String(s) => s,
            _ => panic!(),
        }
    }
}

// Subtype rules
//
// T <= Set(T)
// Void == Set(v) if v.is_empty()
// T <= Query(T)
// Position <= Location
// Range <= Location
//
// Special rules (coercion in evaluation)
//
// Set(T) << T
// Query(T) << T

#[derive(Clone, Eq, PartialEq, Debug)]
pub enum Type {
    Void,
    Query(Box<Type>),
    Number,
    Set(Box<Type>),
    Identifier,
    Location,
    Position,
    Range,
    String,
    Definition,
}

impl Type {
    pub fn is_query(&self) -> bool {
        match self {
            Type::Query(_) => true,
            _ => false,
        }
    }

    pub fn is_location(&self) -> bool {
        match self {
            Type::Location | Type::Position | Type::Range => true,
            Type::Query(ty) | Type::Set(ty) => ty.is_location(),
            _ => false,
        }
    }

    pub fn unquery(&self) -> Type {
        match self {
            Type::Query(inner) => inner.unquery(),
            _ => self.clone(),
        }
    }

    pub fn expect_set_inner(&self) -> Type {
        match self {
            Type::Set(inner) => (**inner).clone(),
            _ => panic!("unexpected: {:?}", self),
        }
    }

    pub fn element_type(&self) -> Option<&Type> {
        match self {
            Type::Set(t) => Some(&t),
            _ => None,
        }
    }
}

impl fmt::Display for Type {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Type::Void => write!(f, "void"),
            Type::Query(t) => write!(f, "query<{}>", t),
            Type::Number => write!(f, "number"),
            Type::Set(t) => write!(f, "set<{}>", t),
            Type::Identifier => write!(f, "identifier"),
            Type::Location => write!(f, "location"),
            Type::Position => write!(f, "position"),
            Type::Range => write!(f, "range"),
            Type::String => write!(f, "string"),
            Type::Definition => write!(f, "def"),
        }
    }
}

#[derive(Clone)]
pub enum ValueKind {
    Void,
    Number(usize),
    Set(Vec<Value>),
    Position(Position),
    Range(Range),
    Query(Query),
    Identifier(Identifier),
    String(String),
    Definition(Definition),
}

impl ValueKind {
    pub fn is_void(&self) -> bool {
        match self {
            ValueKind::Void => true,
            ValueKind::Set(v) if v.is_empty() => true,
            _ => false,
        }
    }
}

impl Show for ValueKind {
    fn show(&self, w: &mut dyn Write, env: &impl Environment) -> Result<(), Error> {
        match self {
            ValueKind::Void => write!(w, "()").map_err(Into::into),
            ValueKind::Number(n) => write!(w, "{}", n).map_err(Into::into),
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
                    write!(w, "]").map_err(Into::into)
                } else {
                    write!(w, "[...]*{}", v.len()).map_err(Into::into)
                }
            }
            ValueKind::Position(p) => p.show(w, env),
            ValueKind::Range(r) => r.show(w, env),
            ValueKind::String(s) => write!(w, "\"{}\"", s).map_err(Into::into),
            ValueKind::Identifier(id) => write!(w, "`{}`", id.name).map_err(Into::into),
            ValueKind::Query(_) => write!(w, "<Query>").map_err(Into::into),
            ValueKind::Definition(def) => {
                write!(w, "`{}` at ", def.name)?;
                def.span.show(w, env)
            }
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Definition {
    pub id: u64,
    pub span: Span,
    pub name: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Identifier {
    pub id: u64,
    pub span: Span,
    pub name: String,
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

#[derive(new, Clone, Debug, Eq, PartialEq)]
pub struct Position {
    pub file: Path,
    pub line: usize,
    pub column: usize,
}

impl Show for Position {
    fn show(&self, w: &mut dyn Write, env: &impl Environment) -> Result<(), Error> {
        write!(w, " --> ")?;
        env.file_system().show_path(self.file, w)?;
        let text = env.file_system().with_file(self.file, |file| {
            file.lines.get(self.line).map(|s| s.to_owned())
        })?;
        write!(w, ":{}:{}\n", self.line + 1, self.column + 1)?;
        write!(
            w,
            "{} | {}\n",
            self.line + 1,
            text.unwrap_or_else(|| "<error - line out of range>".to_owned())
        )?;
        let offset = (self.line + 1).to_string().len() + 3;
        write!(w, "{:width$}^", "", width = offset + self.column).map_err(Into::into)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Range {
    File(Path),
    MultiFile(Vec<Path>),
    Line(Path, usize),
    Span(Span),
}

impl Show for Range {
    fn show(&self, w: &mut dyn Write, env: &impl Environment) -> Result<(), Error> {
        match self {
            Range::File(path) => env.file_system().show_path(*path, w).map_err(Into::into),
            Range::MultiFile(paths) if paths.len() < 5 => {
                write!(w, "[")?;
                let mut first = true;
                for p in paths {
                    if first {
                        first = false;
                    } else {
                        write!(w, ", ")?;
                    }
                    env.file_system().show_path(*p, w)?;
                }
                write!(w, "]").map_err(Into::into)
            }
            Range::MultiFile(paths) => write!(w, "[{} files]", paths.len()).map_err(Into::into),
            Range::Line(path, line) => {
                write!(w, " --> ")?;
                env.file_system().show_path(*path, w)?;
                let text = env
                    .file_system()
                    .with_file(*path, |file| file.lines.get(*line).map(|s| s.to_owned()))?;
                write!(w, ":{}\n", line + 1)?;
                write!(
                    w,
                    "{} | {}",
                    line + 1,
                    text.unwrap_or_else(|| "<error - line out of range>".to_owned())
                )
                .map_err(Into::into)
            }
            Range::Span(s) => s.show(w, env),
        }
    }
}

#[derive(new, Clone, Debug, Eq, PartialEq)]
pub struct Span {
    pub file: Path,
    pub start_line: usize,
    pub start_column: usize,
    pub end_line: usize,
    pub end_column: usize,
}

impl Show for Span {
    fn show(&self, w: &mut dyn Write, env: &impl Environment) -> Result<(), Error> {
        write!(w, " --> ")?;
        env.file_system().show_path(self.file, w)?;
        if self.start_line == self.end_line {
            // A span on one line
            let text = env.file_system().with_file(self.file, |file| {
                file.lines.get(self.start_line).map(|s| s.to_owned())
            })?;
            write!(
                w,
                ":{}:{}->{}\n",
                self.start_line + 1,
                self.start_column + 1,
                self.end_column + 1
            )?;
            write!(
                w,
                "{} | {}\n",
                self.start_line + 1,
                text.unwrap_or_else(|| "<error - line out of range>".to_owned())
            )?;
            let offset = (self.start_line + 1).to_string().len() + 3;
            write!(
                w,
                "{:width1$}{}",
                "",
                "^".repeat(self.end_column - self.start_column),
                width1 = offset + self.start_column
            )
            .map_err(Into::into)
        } else {
            // A multispan range
            write!(
                w,
                ":{}:{}->{}:{}\n",
                self.start_line + 1,
                self.start_column + 1,
                self.end_line + 1,
                self.end_column + 1
            )
            .map_err(Into::into)
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::env::mock::MockEnv;

    #[test]
    fn test_value_show() {
        assert_eq!(Value::void().show_str(&MockEnv), "()");
        assert_eq!(Value::number(42).show_str(&MockEnv), "42");
        let set = Value {
            kind: ValueKind::Set(vec![Value::number(1), Value::number(2), Value::number(3)]),
            ty: Type::Set(Box::new(Type::Number)),
        };
        assert_eq!(set.show_str(&MockEnv), "[1, 2, 3]");
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
        assert_eq!(set.show_str(&MockEnv), "[...]*8");
    }

    #[test]
    fn test_location_show() {
        let env = MockEnv;
        let fs = env.file_system();

        let pos = Position::new(
            fs.find("foo.rs".to_owned().into()).unwrap().pop().unwrap(),
            2,
            3,
        );
        let s = pos.show_str(&env);
        assert!(s.contains("foo.rs:3"));
        assert!(s.contains("This is line 2 of a file with number 1."));

        let range = Range::Line(
            fs.find("foo.rs".to_owned().into()).unwrap().pop().unwrap(),
            3,
        );
        let s = range.show_str(&env);
        assert!(s.contains("foo.rs:4"));
        assert!(s.contains("This is line 3 of a file with number 1."));

        let span = Span::new(
            fs.find("foo.rs".to_owned().into()).unwrap().pop().unwrap(),
            3,
            1,
            3,
            10,
        );
        let s = span.show_str(&env);
        assert!(s.contains("foo.rs:4:2->11"));
        assert!(s.contains("This is line 3 of a file with number 1."));
    }
}

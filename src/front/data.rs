use crate::file_system::Path;

#[derive(Clone, Eq, PartialEq, Debug)]
pub struct Position {
    pub file: Path,
    pub line: usize,
    pub column: usize,
}

#[derive(Clone, Eq, PartialEq, Debug)]
pub enum Range {
    File(Path),
    MultiFile(Vec<Path>),
    Line(Path, usize),
    Span(Span),
}

#[derive(Clone, Eq, PartialEq, Debug)]
pub struct Span {
    pub file: Path,
    pub start_line: usize,
    pub start_column: usize,
    pub end_line: usize,
    pub end_column: usize,
}

use super::{Backend, Error};
use crate::file_system::{FileSystem, PhysicalFs};
use crate::front::data::{Identifier, Position, Range, Span};

use rls_analysis::{AnalysisHost, Id, Ident, Span as RlsSpan, Target};
use rls_span::{Column, Row};
use std::mem;
use std::process::Command;
use std::rc::Rc;

// FIXME use `join` not `/`
const TARGET_DIR: &str = "target/rls";

pub struct Rls<Fs: FileSystem> {
    analysis_host: AnalysisHost,
    fs: Rc<Fs>,
}

impl Rls<PhysicalFs> {
    pub fn init(fs: Rc<PhysicalFs>) -> Rls<PhysicalFs> {
        let analysis_host = AnalysisHost::new(Target::Debug);
        println!("building index");
        Self::reindex();
        println!("loading analysis...");
        // TODO use blacklist
        analysis_host.reload(&fs.root, &fs.root).unwrap();
        Rls { analysis_host, fs }
    }

    fn reindex() {
        // FIXME redirect stdout to a log file
        // FIXME set the base directory according to the root of the fs
        let mut cmd = Command::new("cargo");
        cmd.arg("check");
        // FIXME configure save-analysis
        cmd.env("RUSTFLAGS", "-Zunstable-options -Zsave-analysis");
        cmd.env("CARGO_TARGET_DIR", TARGET_DIR);

        let status = cmd.status().expect("Running build failed");
        // FIXME handle an error instead of unwrapping
        let result = status.code().unwrap();
        // FIXME cleanup analysis (see cargo src)
    }
}

impl<Fs: FileSystem> Backend for Rls<Fs> {
    fn ident_at(&self, position: Position) -> Result<Option<Identifier>, Error> {
        let idents = self.analysis_host.idents(&position.into_with(&*self.fs)?)?;
        Ok(match idents.into_iter().next() {
            Some(i) => Some(i.into_with(&*self.fs)?),
            None => None,
        })
    }

    fn idents_in(&self, range: Range) -> Result<Vec<Identifier>, Error> {
        let idents = self.analysis_host.idents(&range.into_with(&*self.fs)?)?;
        idents.into_iter().map(|i| i.into_with(&*self.fs)).collect()
    }
}

trait IntoWithFs<T, Fs: FileSystem> {
    fn into_with(self, fs: &Fs) -> Result<T, Error>;
}

impl<Fs: FileSystem> IntoWithFs<RlsSpan, Fs> for Position {
    fn into_with(self, fs: &Fs) -> Result<RlsSpan, Error> {
        let row = Row::new_zero_indexed(self.line as u32);
        let column = Column::new_zero_indexed(self.column as u32);
        Ok(RlsSpan::new(
            row,
            row,
            column,
            column,
            fs.physical_path(&self.file)?,
        ))
    }
}

impl<Fs: FileSystem> IntoWithFs<RlsSpan, Fs> for Range {
    fn into_with(self, fs: &Fs) -> Result<RlsSpan, Error> {
        match self {
            Range::Line(p, line) => {
                let row = Row::new_zero_indexed(line as u32);
                let column_start = Column::new_zero_indexed(0);
                let column_end = Column::new_zero_indexed(255);
                Ok(RlsSpan::new(
                    row,
                    row,
                    column_start,
                    column_end,
                    fs.physical_path(&p)?,
                ))
            }
            Range::Span(sp) => sp.into_with(fs),
            r => Err(Error::Back(format!("Unimplemented range: {:?}", r))),
        }
    }
}

impl<Fs: FileSystem> IntoWithFs<RlsSpan, Fs> for Span {
    fn into_with(self, fs: &Fs) -> Result<RlsSpan, Error> {
        Ok(RlsSpan::new(
            Row::new_zero_indexed(self.start_line as u32),
            Row::new_zero_indexed(self.end_line as u32),
            Column::new_zero_indexed(self.start_column as u32),
            Column::new_zero_indexed(self.end_column as u32),
            fs.physical_path(&self.file)?,
        ))
    }
}

impl<Fs: FileSystem> IntoWithFs<Identifier, Fs> for Ident {
    fn into_with(self, fs: &Fs) -> Result<Identifier, Error> {
        let span = self.span.into_with(fs)?;
        Ok(Identifier {
            id: unsafe { mem::transmute::<Id, u64>(self.id) },
            name: fs.snippet(&Range::Span(span.clone()))?,
            span,
        })
    }
}

impl<Fs: FileSystem> IntoWithFs<Span, Fs> for RlsSpan {
    fn into_with(self, fs: &Fs) -> Result<Span, Error> {
        Ok(Span::new(
            fs.resolve_path(&self.file)?,
            self.range.row_start.0 as usize,
            self.range.row_end.0 as usize,
            self.range.col_start.0 as usize,
            self.range.col_end.0 as usize,
        ))
    }
}

impl From<rls_analysis::AError> for Error {
    fn from(e: rls_analysis::AError) -> Error {
        Error::Back(format!("Error in RLS backend: {}", e))
    }
}

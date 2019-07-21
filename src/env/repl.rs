use super::Environment;
use crate::parse;
use std::cell::Cell;
use std::env;
use std::io::{stdin, stdout, Write};
use std::path::PathBuf;

pub struct Repl {
    config: Config,
    line_count: Cell<usize>,
}

impl Repl {
    pub fn new(config: Config) -> Repl {
        Repl {
            config,
            line_count: Cell::new(0),
        }
    }

    pub fn run(&self) {
        let stdin = stdin();
        let mut buf = String::new();
        loop {
            let prompt = self.prompt();
            print!("{}", prompt);
            stdout().flush().expect("Couldn't flush stdout");
            buf.truncate(0);
            stdin.read_line(&mut buf).expect("Error reading from stdin");
            match parse::parse_stmt(&buf, None) {
                Ok(node) => {
                    self.incr_line_count();
                    // TODO execute node
                }
                Err(e) => match e {
                    parse::Error::EmptyInput => {}
                    parse::Error::Lexing(msg, offset) => {
                        let offset = offset + prompt.len();
                        println!("{}^", " ".repeat(offset));
                        println!("{}", msg);
                        self.incr_line_count();
                    }
                    parse::Error::Parsing(msg) => {
                        println!("{}", msg);
                        self.incr_line_count();
                    }
                    parse::Error::Other(msg) => println!("Error parsing input: {}", msg),
                },
            }
        }
    }

    fn prompt(&self) -> String {
        format!("{} > ", self.line_count.get())
    }

    fn incr_line_count(&self) {
        let line_count = self.line_count.get();
        self.line_count.set(line_count + 1);
    }
}

impl Environment for Repl {
    type ParseContext = ReplParseContext;
}

pub struct Config {
    current_dir: PathBuf,
}

impl Default for Config {
    fn default() -> Config {
        Config {
            current_dir: env::current_dir().expect("Could not access current directory"),
        }
    }
}

#[derive(Clone)]
pub struct ReplParseContext {
    line_number: usize,
}

impl parse::EnvContext for ReplParseContext {
    fn clone(&self) -> Box<dyn parse::EnvContext> {
        Box::new(Clone::clone(self))
    }
}

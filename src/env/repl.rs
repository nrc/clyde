use super::Environment;
use crate::file_system::PhysicalFs;
use crate::front::{self, Show};
use crate::parse::{self, ast};
use std::cell::Cell;
use std::env;
use std::io::{stdin, stdout, Write};
use std::path::PathBuf;
use std::process;

pub struct Repl {
    config: Config,
    line_count: Cell<usize>,
    file_system: PhysicalFs,
}

impl Repl {
    pub fn new(config: Config) -> Repl {
        Repl {
            file_system: PhysicalFs::new(&config.current_dir),
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
                    let interpreter = front::Interpreter::new(self);
                    if let Err(e) = interpreter.interpret(node) {
                        println!("{}", e);
                    }
                    self.incr_line_count();
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
    type Fs = PhysicalFs;

    fn exec_meta(&self, mk: ast::MetaKind) -> Result<(), front::Error> {
        match mk {
            ast::MetaKind::Exit => process::exit(0),
            ast::MetaKind::Help => {
                println!("Clyde 0.1");
                println!("");
                println!("Meta-commands:");
                println!("  ^help     display this message");
                println!("  ^exit     exit Clyde");
                println!("");
                println!("Some common statements:");
                println!("  select    query the program");
                println!("  x =       variable assignment");
                println!("  show      print a value");
            }
        }

        Ok(())
    }

    fn show(&self, s: &impl Show) -> Result<(), front::Error> {
        println!("{}", s.to_string(self));
        Ok(())
    }

    fn lookup_var(&self, var: &front::MetaVar) -> Result<front::Value, front::Error> {
        // FIXME lookup variable by name
        Err(front::Error::VarNotFound(var.clone()))
    }
    fn lookup_numeric_var(&self, id: isize) -> Result<front::Value, front::Error> {
        unimplemented!();
    }

    fn file_system(&self) -> &PhysicalFs {
        &self.file_system
    }
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

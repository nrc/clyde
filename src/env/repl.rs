use super::Environment;
use crate::back;
use crate::file_system::PhysicalFs;
use crate::front::{self, data, MetaVar, Show};
use crate::parse::{self, ast};
use std::cell::RefCell;
use std::env;
use std::io::{stdin, stdout, Write};
use std::path::PathBuf;
use std::process;
use std::rc::Rc;

pub struct Repl {
    config: Config,
    file_system: Rc<PhysicalFs>,
    rls: RefCell<Option<Rc<back::Rls<PhysicalFs>>>>,
    prev_results: RefCell<Vec<Option<data::Value>>>,
}

impl Repl {
    pub fn new(config: Config) -> Repl {
        Repl {
            file_system: Rc::new(PhysicalFs::new(&config.current_dir)),
            config,
            rls: RefCell::new(None),
            prev_results: RefCell::new(Vec::new()),
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
                    let result = self.interpret(node);
                }
                Err(e) => match e {
                    parse::Error::EmptyInput => {}
                    parse::Error::Lexing(msg, offset) => {
                        let offset = offset + prompt.len();
                        println!("{}^", " ".repeat(offset));
                        println!("{}", msg);
                        self.prev_results.borrow_mut().push(None);
                    }
                    parse::Error::Parsing(msg) => {
                        println!("{}", msg);
                        self.prev_results.borrow_mut().push(None);
                    }
                    parse::Error::Other(msg) => println!("Error parsing input: {}", msg),
                },
            }
        }
    }

    fn interpret(&self, stmt: ast::Statement) -> Result<front::Value, front::Error> {
        let mut interpreter = front::Interpreter::new(self);
        let result = interpreter.interpret_stmt(stmt.clone());
        match &result {
            Ok(v) => self.prev_results.borrow_mut().push(Some(v.clone())),
            Err(e) => {
                println!("Error: {}", e);
                self.prev_results.borrow_mut().push(None);
            }
        }
        result
    }

    fn prompt(&self) -> String {
        format!("{} > ", self.prev_results.borrow().len())
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
        println!("{}", s.show_str(self));
        Ok(())
    }

    fn lookup_var(&self, var: &front::MetaVar) -> Result<front::Value, front::Error> {
        // TODO lookup variable by name
        Err(front::Error::VarNotFound(var.clone()))
    }

    fn lookup_numeric_var(&self, mut id: isize) -> Result<front::Value, front::Error> {
        let prev_result = {
            let prev_results = self.prev_results.borrow();
            if id < 0 {
                id = prev_results.len() as isize + id;
            }
            if id < 0 || id as usize >= prev_results.len() {
                return Err(front::Error::NumericVarNotFound(
                    id as usize,
                    prev_results.len().saturating_sub(1),
                ));
            }
            prev_results[id as usize].clone()
        };
        if let Some(result) = prev_result {
            Ok(result)
        } else {
            Err(front::Error::VarNotFound(MetaVar::new(&id.to_string())))
        }
    }

    fn file_system(&self) -> &PhysicalFs {
        &self.file_system
    }

    fn backend(&self) -> Rc<dyn back::Backend> {
        let mut rls = self.rls.borrow_mut();
        match &*rls {
            Some(rls) => rls.clone(),
            None => {
                *rls = Some(Rc::new(back::Rls::init(self.file_system.clone())));
                rls.as_ref().unwrap().clone()
            }
        }
    }
}

pub struct Config {
    pub current_dir: PathBuf,
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

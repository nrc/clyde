use clyde::{Repl, ReplConfig};

fn main() {
    let config = ReplConfig::default();
    let repl = Repl::new(config);
    repl.run();
}

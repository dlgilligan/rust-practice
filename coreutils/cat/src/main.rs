use std::env;
use std::process;

use cat::run;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Too few arguments");
    } else if args.len() > 2 {
        eprintln!("Too many arguments");
    }

    let file = &args[1];

    if let Err(e) = run(file) {
        eprintln!("Error reading file: {e}");
        process::exit(1)
    };
}

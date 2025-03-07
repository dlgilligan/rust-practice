use clap::Parser;
use std::fs::File;
use std::io::BufRead;
use std::io::BufReader;
use std::process;

#[derive(Parser)]
#[command(name = "head")]
#[command(about = "Displays file contents line-by-line, from the start of the file")]
pub struct Args {
    file: String,

    #[arg(short = 'n', long, default_value = "10")]
    lines: usize,
}

fn main() {
    let args = Args::parse();

    let file = File::open(args.file).unwrap_or_else(|err| {
        eprintln!("Failed to open file: {err}");
        process::exit(1);
    });
    let reader = BufReader::new(&file);

    for (num, line) in reader.lines().enumerate() {
        if num == args.lines {
            process::exit(0);
        }

        match line {
            Ok(x) => println!("{x}"),
            Err(_) => process::exit(0),
        }
    }
}

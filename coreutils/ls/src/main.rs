use clap::Parser;
use std::error::Error;
use std::fs;

#[derive(Parser)]
#[command(name = "ls")]
#[command(about = "Lists directory contents")]
pub struct Args {
    #[clap(default_value = "./", num_args(1..))]
    dests: Vec<String>,

    #[arg(short, long)]
    all: bool,
}

pub fn ls(args: Args) -> Result<(), Box<dyn Error>> {
    for dir in &args.dests {
        let dir_contents = fs::read_dir(dir)?;

        if args.dests.len() > 1 {
            println!("{dir}:");
        }

        for entry in dir_contents {
            let file_name = entry.unwrap().file_name().into_string().unwrap();

            if !file_name.starts_with(".") {
                print!("{file_name}  ");
            } else if args.all {
                print!("{file_name}  ");
            }
        }
        println!("\n");
    }

    Ok(())
}

fn main() {
    let args = Args::parse();

    if let Err(e) = ls(args) {
        eprintln!("Error listing directory: {e}");
    }
}

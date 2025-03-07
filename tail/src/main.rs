use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader, Read, Seek, SeekFrom};
use std::process;

use clap::Parser;

#[derive(Parser)]
#[command(name = "tail")]
#[command(about = "Displays file contents from the end of the file")]
pub struct Args {
    file: String,

    #[arg(short = 'n', long, default_value = "10")]
    lines: usize,
}

fn main() {
    let args = Args::parse();

    if let Err(e) = read_from_end(args) {
        eprintln!("{e}");
        process::exit(1);
    }
}

fn read_from_end(args: Args) -> Result<(), Box<dyn Error>> {
    let mut file = File::open(&args.file)?;
    let file_size = file.metadata()?.len() as usize;

    // If the file is empty, return early
    if file_size == 0 {
        return Ok(());
    }

    // We need to find the start of the Nth line from the end
    let mut buffer = [0; 4096];
    let mut newline_count = 0;
    let mut position = file_size;

    // Count newlines from the end
    while position > 0 && newline_count <= args.lines {
        let bytes_to_read = std::cmp::min(position, buffer.len());
        position -= bytes_to_read;

        file.seek(SeekFrom::Start(position as u64))?;
        let bytes_read = file.read(&mut buffer[..bytes_to_read])?;

        for i in (0..bytes_read).rev() {
            if buffer[i] == b'\n' {
                newline_count += 1;
                if newline_count > args.lines {
                    // We found one more newline than needed - this is our starting point
                    position += i + 1; // Start after this newline
                    break;
                }
            }
        }

        if newline_count > args.lines {
            break;
        }
    }

    // Position is now at the start of the lines we want to print
    file.seek(SeekFrom::Start(position as u64))?;

    // Read each line and print it (to avoid any issues with newlines)
    let mut reader = BufReader::new(file);
    let mut line = String::new();

    while reader.read_line(&mut line)? > 0 {
        // Avoid printing the trailing newline on the last line
        print!("{}", line);
        line.clear();
    }

    Ok(())
}

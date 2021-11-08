#![warn(clippy::all, clippy::pedantic)]
use std::convert::TryFrom;
use std::env;
use std::fs::File;
use std::io;
use std::io::{Read, Stdin};
use std::process;
use thiserror::Error;

/// `PassedFile` represents some kind of file that will be passed in an argument
enum PassedFile {
    Stdin,
    Path(String),
}

/// `OpenedFile` represents some kind of file that was opened for further handling by `hl`
enum OpenedFile {
    Stdin(Stdin),
    File(File),
}

/// `Args` represents arguments passed to the program
struct Args {
    pattern: String,
    file: PassedFile,
}

/// `ArgsError` represents an error encountered while passing [Args]
#[derive(Error, Debug)]
enum ArgsError {
    #[error("wrong number of arguments")]
    WrongNumber,
}

impl Read for OpenedFile {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        match self {
            // TODO: If more variants are ever added this could probably be a macro
            Self::Stdin(read) => read.read(buf),
            Self::File(read) => read.read(buf),
        }
    }
}

impl TryFrom<env::Args> for Args {
    type Error = ArgsError;

    fn try_from(args: env::Args) -> Result<Self, Self::Error> {
        let mut program_args = args.skip(1);
        let pattern = {
            if let Some(pattern) = program_args.next() {
                pattern
            } else {
                return Err(ArgsError::WrongNumber);
            }
        };

        let file = program_args
            .next()
            .map_or_else(|| PassedFile::Stdin, PassedFile::Path);

        Ok(Args { pattern, file })
    }
}

fn main() {
    let args_parse_result = Args::try_from(env::args());
    if let Err(ArgsError::WrongNumber) = args_parse_result {
        print_usage();
        process::exit(1);
    }

    let args = args_parse_result.unwrap();
    let open_file_result = open_file(args.file);
    if let Err(err) = open_file_result {
        eprintln!("Failed to open input file: {}", err);
        process::exit(2);
    }

    let opened_file = open_file_result.unwrap();
    let scan_result = hline::scan_pattern(opened_file, &args.pattern);
    if let Err(err) = scan_result {
        eprintln!("Failed to open scan file: {}", err);
        process::exit(3);
    }
}

fn open_file(file: PassedFile) -> Result<OpenedFile, std::io::Error> {
    match file {
        PassedFile::Stdin => Ok(OpenedFile::Stdin(std::io::stdin())),
        PassedFile::Path(path) => {
            let file = File::open(path)?;
            Ok(OpenedFile::File(file))
        }
    }
}

fn print_usage() {
    let program_name = env::args().next().unwrap_or_else(|| "hl".to_string());
    eprintln!("Usage: {} pattern [file]", program_name);
}

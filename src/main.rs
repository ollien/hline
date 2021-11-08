#![warn(clippy::all, clippy::pedantic)]
use clap::{crate_name, crate_version, App, AppSettings, Arg, ArgMatches};
use std::env;
use std::fs::File;
use std::io;
use std::io::{Read, Stdin};
use std::process;

const FILENAME_ARG_NAME: &str = "filename";
const PATTERN_ARG_NAME: &str = "pattern";

/// `OpenedFile` represents some kind of file that was opened for further handling by `hl`
enum OpenedFile {
    Stdin(Stdin),
    File(File),
}

/// `PassedFile` represents some kind of file that will be passed in an argument
enum PassedFile {
    Stdin,
    Path(String),
}

/// `Args` represents arguments passed to the program
struct Args {
    pattern: String,
    file: PassedFile,
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

impl From<ArgMatches<'_>> for Args {
    fn from(args: ArgMatches) -> Self {
        let pattern = args
            .value_of(PATTERN_ARG_NAME)
            .expect("pattern arg not found, despite parser reporting it was present")
            .to_string();

        let file = args
            .value_of(FILENAME_ARG_NAME)
            .map_or(PassedFile::Stdin, |filename| {
                PassedFile::Path(filename.to_string())
            });

        Args { pattern, file }
    }
}

fn main() {
    let parsed_args = setup_arg_parser().get_matches();
    let args_parse_result = Args::try_from(parsed_args);

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

/// Setup the argument parser for the program with all possible flags
fn setup_arg_parser() -> App<'static, 'static> {
    App::new(crate_name!())
        .version(crate_version!())
        .about("Highlights lines that match the given regular expression")
        .setting(AppSettings::DisableVersion)
        .arg(
            Arg::with_name("pattern")
                .takes_value(true)
                .required(true)
                .allow_hyphen_values(true)
                .help(concat!(
                    "The regular expression to search for. Note that this is not anchored, and if ",
                    "anchoring is desired, should be done manually with ^ or $."
                )),
        )
        .arg(
            Arg::with_name(FILENAME_ARG_NAME)
                .takes_value(true)
                .help("The file to scan. If not specified, reads from stdin"),
        )
}

/// Open the file that was passed to the command line
fn open_file(file: PassedFile) -> Result<OpenedFile, std::io::Error> {
    match file {
        PassedFile::Stdin => Ok(OpenedFile::Stdin(std::io::stdin())),
        PassedFile::Path(path) => {
            let file = File::open(path)?;
            Ok(OpenedFile::File(file))
        }
    }
}

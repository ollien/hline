#![warn(clippy::all, clippy::pedantic)]
use grep::regex;
use grep::regex::RegexMatcher;
use grep::searcher::SearcherBuilder;
use print::{Printer, StdoutPrinter};
use std::io;
use std::io::Read;
use thiserror::Error;

mod lines;
pub mod print;
mod sink;

#[cfg(test)]
mod testutil;

/// Error represents the possible errors that can occur during the search process. See `scan_pattern` for more details.
#[derive(Error, Debug)]
pub enum Error {
    #[error("{0}")]
    RegexError(regex::Error),
    #[error("Search process encountered a failure: {0}")]
    SearchError(String),
    #[error("Print failure: {0}")]
    PrintFailure(io::Error),
}

impl From<sink::Error> for Error {
    fn from(err: sink::Error) -> Self {
        match err {
            sink::Error::SearchError(msg) => Error::SearchError(msg),
            sink::Error::PrintFailed(io_err) => Error::PrintFailure(io_err),
        }
    }
}

impl From<regex::Error> for Error {
    fn from(err: regex::Error) -> Self {
        Self::RegexError(err)
    }
}

/// `scan_pattern` will print a reader's contents, while also scanning its contents for a regular expression.
/// Lines that match this pattern will be highlighted in the output.
/// A convenience wrapper for `scan_pattern_to_printer` that will print to stdout.
///
/// # Errors
///
/// See `scan_pattern_to_printer`
pub fn scan_pattern<R: Read>(reader: R, pattern: &str) -> Result<(), Error> {
    scan_pattern_to_printer(reader, pattern, StdoutPrinter::new())
}

/// `scan_pattern_to_printer` will print a `Read`'s contents to the given `Printer`, while also scanning its contents
/// for a regular expression. Lines that match this pattern will be highlighted in the output.
///
/// Note that this pattern is not anchored at the start of the line by default, and therefore a match anywhere in the
/// line will force the entire line to be considered a match. For instance, the pattern `[a-z]` will match `123abc456`.
///
/// # Errors
///
/// There are three general error cases
/// - An invalid reuglar expression
/// - I/O errors in scanning from the `Read`
/// - An error produced by the underlying grep library during the search
pub fn scan_pattern_to_printer<R: Read, P: Printer>(
    reader: R,
    pattern: &str,
    printer: P,
) -> Result<(), Error> {
    let matcher = RegexMatcher::new(pattern)?;
    let mut searcher = SearcherBuilder::new().passthru(true).build();
    let context_sink = sink::ContextPrintingSink::new(printer);

    searcher.search_reader(matcher, reader, context_sink)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testutil;
    use std::io;
    use stringreader::StringReader;
    use test_case::test_case;
    use testutil::mock_print::MockPrinter;

    const SEARCH_TEXT: &str = "The \"computable\" numbers may be described briefly as the real \n\
    numbers whose expressions as a decimal are calculable by finite means. \n\
    Although the subject of this paper is ostensibly the computable numbers. \n\
    it is almost equally easy to define and investigate computable functions \n\
    of an integral variable or a real or computable variable, computable \n\
    predicates, and so forth. The fundamental problems involved are, \n\
    however, the same in each case, and I have chosen the computable numbers \n\
    for explicit treatment as involving the least cumbrous technique. I hope \n\
    shortly to give an account of the relations of the computable numbers, \n\
    functions, and so forth to one another. This will include a development \n\
    of the theory of functions of a real variable expressed in terms of \n\
    computable numbers. According to my definition, a number is computable \n\
    if its decimal can be written down by a machine.\n";

    #[test]
    fn test_highlights_matches() {
        let mock_printer = MockPrinter::default();
        let mut lipsum_reader = StringReader::new(SEARCH_TEXT);
        let res = scan_pattern_to_printer(
            &mut lipsum_reader,
            r#""?computable"?\snumbers"#,
            &mock_printer,
        );
        if let Err(err) = res {
            panic!("failed to search: {}", err)
        }

        let colored_messages = mock_printer.colored_messages.borrow();
        #[rustfmt::skip]
        let expected_colored_messages = [
            "The \"computable\" numbers may be described briefly as the real \n".to_string(),
            "Although the subject of this paper is ostensibly the computable numbers. \n".to_string(),
            "however, the same in each case, and I have chosen the computable numbers \n".to_string(),
            "shortly to give an account of the relations of the computable numbers, \n".to_string(),
            "computable numbers. According to my definition, a number is computable \n".to_string(),
        ];
        testutil::assert_slices_eq!(&colored_messages, &expected_colored_messages);

        let uncolored_messages = mock_printer.uncolored_messages.borrow();
        #[rustfmt::skip]
        let expected_uncolored_messages = [
            "numbers whose expressions as a decimal are calculable by finite means. \n".to_string(),
            "it is almost equally easy to define and investigate computable functions \n".to_string(),
            "of an integral variable or a real or computable variable, computable \n".to_string(),
            "predicates, and so forth. The fundamental problems involved are, \n".to_string(),
            "for explicit treatment as involving the least cumbrous technique. I hope \n".to_string(),
            "functions, and so forth to one another. This will include a development \n".to_string(),
            "of the theory of functions of a real variable expressed in terms of \n".to_string(),
            "if its decimal can be written down by a machine.\n".to_string(),
        ];
        testutil::assert_slices_eq!(&uncolored_messages, &expected_uncolored_messages);
    }

    #[test]
    fn case_insensitive_pattern_matches() {
        let mock_printer = MockPrinter::default();
        let mut lipsum_reader = StringReader::new(SEARCH_TEXT);
        // This test is a little bit of a cheat, because it doesn't test what's actually inputted by the CLI,
        // but it does make sure the functionality works as expected
        let res = scan_pattern_to_printer(&mut lipsum_reader, "(?i)INTEGRAL", &mock_printer);
        if let Err(err) = res {
            panic!("failed to search: {}", err)
        }

        let colored_messages = mock_printer.colored_messages.borrow();
        let expected_colored_messages = [
            "of an integral variable or a real or computable variable, computable \n".to_string(),
        ];
        testutil::assert_slices_eq!(&colored_messages, &expected_colored_messages);

        let uncolored_messages = mock_printer.uncolored_messages.borrow();
        // Again, the only missing message is the previous
        #[rustfmt::skip]
        let expected_uncolored_messages = [
            "The \"computable\" numbers may be described briefly as the real \n".to_string(),
            "numbers whose expressions as a decimal are calculable by finite means. \n".to_string(),
            "Although the subject of this paper is ostensibly the computable numbers. \n".to_string(),
            "it is almost equally easy to define and investigate computable functions \n".to_string(),
            "predicates, and so forth. The fundamental problems involved are, \n".to_string(),
            "however, the same in each case, and I have chosen the computable numbers \n".to_string(),
            "for explicit treatment as involving the least cumbrous technique. I hope \n".to_string(),
            "shortly to give an account of the relations of the computable numbers, \n".to_string(),
            "functions, and so forth to one another. This will include a development \n".to_string(),
            "of the theory of functions of a real variable expressed in terms of \n".to_string(),
            "computable numbers. According to my definition, a number is computable \n".to_string(),
            "if its decimal can be written down by a machine.\n".to_string()
        ];
        testutil::assert_slices_eq!(&uncolored_messages, &expected_uncolored_messages);
    }

    #[test_case(".", 0, 1; "failure on first match will only attempt to print that match")]
    #[test_case("hello I am alan turing", 1, 0; "never matching will only attempt to print the first line")]
    fn test_does_not_attempt_to_print_after_broken_pipe_error(
        pattern: &str,
        num_uncolored_messages: usize,
        num_colored_messages: usize,
    ) {
        let mut mock_printer = MockPrinter::default();
        let broken_pipe_err =
            print::Error::from(io::Error::new(io::ErrorKind::BrokenPipe, "broken pipe"));
        mock_printer.fail_next(broken_pipe_err);
        let mut lipsum_reader = StringReader::new(SEARCH_TEXT);
        let res = scan_pattern_to_printer(&mut lipsum_reader, pattern, &mock_printer);

        assert!(!res.is_err(), "failed to search: {:?}", res.unwrap_err());
        assert_eq!(
            num_colored_messages,
            mock_printer.colored_messages.borrow().len()
        );
        assert_eq!(
            num_uncolored_messages,
            mock_printer.uncolored_messages.borrow().len()
        );
    }
}

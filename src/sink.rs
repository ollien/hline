//! `sink` provides utilities to handle the search results provided by `grep`.
use crate::print;
use crate::print::{Printer, StdoutPrinter};
use grep::searcher::{Searcher, Sink, SinkContext, SinkError, SinkMatch};
use std::fmt::Display;
use std::io;
use std::panic;
use termion::color::{Fg, LightRed};
use thiserror::Error;

const PASSTHRU_PANIC_MSG: &str = "passthru is not enabled on the given searcher";

pub(crate) struct ContextPrintingSink<P: Printer> {
    printer: P,
}

/// `Error` represents an error that happens during the search process
///
#[derive(Error, Debug)]
pub enum Error {
    /// Printing to the given printer failed due to an i/o error.
    #[error("Print failure: {0}")]
    PrintFailed(
        /// The original i/o error that caused the print failure.
        io::Error,
    ),

    /// The `SearchError` variant is specifically used to represent errors reported by the internal grep library, and
    /// their reasons may not be specifically matchable as a result.
    // This error is a bit custom, and is intended to be produced by callers on the SinkError trait. As such,
    // the message is just a passthrough
    #[error("{0}")]
    SearchError(
        /// An error message provided by the underlying grep library.
        String,
    ),
}

impl From<print::Error> for Error {
    fn from(err: print::Error) -> Self {
        let io_err = match err {
            print::Error::BrokenPipe(wrapped) | print::Error::Other(wrapped) => wrapped,
        };

        Error::PrintFailed(io_err)
    }
}

impl SinkError for Error {
    fn error_message<T: Display>(message: T) -> Self {
        Error::SearchError(message.to_string())
    }
}

impl<P: Printer> ContextPrintingSink<P> {
    fn get_sink_result_for_print_result(res: print::Result) -> Result<bool, Error> {
        match res {
            Err(print::Error::Other(_)) => Err(Error::from(res.unwrap_err())),
            // It is not an error case to have a broken pipe; it just means we can't output anything more and we
            // shouldn't keep searching
            Err(print::Error::BrokenPipe(_)) => Ok(false),
            Ok(_) => Ok(true),
        }
    }
}

impl<P: Printer> ContextPrintingSink<P> {
    #[must_use]
    pub fn new(printer: P) -> Self {
        ContextPrintingSink { printer }
    }

    fn validate_searcher(searcher: &Searcher) {
        if !searcher.passthru() {
            // We cannot operate normally if this happens
            panic!("{}", PASSTHRU_PANIC_MSG)
        }
    }
}

impl Default for ContextPrintingSink<StdoutPrinter> {
    fn default() -> Self {
        ContextPrintingSink {
            printer: StdoutPrinter {},
        }
    }
}

impl<P: Printer> Sink for ContextPrintingSink<P> {
    type Error = Error;

    fn matched(
        &mut self,
        searcher: &Searcher,
        sink_match: &SinkMatch,
    ) -> Result<bool, Self::Error> {
        Self::validate_searcher(searcher);

        let print_res = self.printer.colored_print(
            Fg(LightRed),
            std::str::from_utf8(sink_match.bytes()).unwrap(),
        );

        Self::get_sink_result_for_print_result(print_res)
    }

    fn context(
        &mut self,
        searcher: &Searcher,
        context: &SinkContext<'_>,
    ) -> Result<bool, Self::Error> {
        Self::validate_searcher(searcher);

        let data = std::str::from_utf8(context.bytes()).unwrap();
        let print_res = self.printer.print(data);

        Self::get_sink_result_for_print_result(print_res)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testutil::mock_print::MockPrinter;
    use grep::regex::RegexMatcher;
    use grep::searcher::SearcherBuilder;
    use test_case::test_case;

    const SEARCH_TEXT: &str = "The quick \n\
    brown fox \n\
    jumped over \n\
    the lazy \n\
    dog.";

    // TODO: This is a bit overkill for a single setting, and could probably be simplified
    enum RequiredSearcherSettings {
        Passthru,
    }

    #[test_case(&[RequiredSearcherSettings::Passthru], true; "passthru")]
    #[test_case(&[], false; "none")]
    fn test_requires_properly_configured_searcher(
        settings: &[RequiredSearcherSettings],
        valid: bool,
    ) {
        // This must be wrapped so we can safely use `panic::catch_unwind`
        let perform_search = || {
            let matcher = RegexMatcher::new("fox").expect("regexp doesn't compile");

            let mock_printer = MockPrinter::default();
            let sink = ContextPrintingSink {
                printer: &mock_printer,
            };

            let mut builder = SearcherBuilder::new();
            for setting in settings {
                match setting {
                    RequiredSearcherSettings::Passthru => builder.passthru(true),
                };
            }

            let mut searcher = builder.build();
            searcher.search_slice(matcher, SEARCH_TEXT.as_bytes(), sink)
        };

        if valid {
            let search_res = perform_search();
            assert!(search_res.is_ok());
        } else {
            let search_res = panic::catch_unwind(perform_search);
            assert!(search_res.is_err());
            // This is a bit brittle, but because we must perform the wrap above to safely catch the panic, it's
            // our best option
            match search_res.unwrap_err().downcast_ref::<String>() {
                Some(err) => assert_eq!(err, PASSTHRU_PANIC_MSG),
                None => panic!("Panicked error was not of expected type"),
            };
        }
    }
}

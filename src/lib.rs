#![warn(clippy::all, clippy::pedantic)]
use grep::regex;
use grep::regex::RegexMatcher;
use grep::searcher::SearcherBuilder;
use print::{Printer, StdoutPrinter};
use std::io;
use std::io::Read;
use thiserror::Error;

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
pub fn scan_pattern<R: Read>(reader: &mut R, pattern: &str) -> Result<(), Error> {
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
	reader: &mut R,
	pattern: &str,
	printer: P,
) -> Result<(), Error> {
	let matcher = RegexMatcher::new(pattern)?;
	let mut searcher = SearcherBuilder::new().passthru(true).build();
	let context_sink = sink::ContextPrintingSink::new(printer);

	searcher.search_reader(matcher, reader, context_sink)?;
	Ok(())
}

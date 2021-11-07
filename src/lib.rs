#![warn(clippy::all, clippy::pedantic)]
use grep::regex;
use grep::regex::RegexMatcher;
use grep::searcher::SearcherBuilder;
use std::io;
use std::io::Read;
use thiserror::Error;

mod print;
mod sink;

/// Error represents the possible errors that can occur during the search process. See [scan_pattern] for more details.
#[derive(Error, Debug)]
pub enum Error {
	#[error("{0}")]
	RegexError(regex::Error),
	#[error("Search process encountedered a failure: {0}")]
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
///
/// Note that this pattern is not anchored at the start of the line by default, and therefore a match anywhere in the
/// line will force the entire line to be considered a match. For instance, the pattern `[a-z]` will match `123abc456`.
///
/// # Errors
///
/// There are three general error cases
/// - An invalid reuglar expression
/// - I/O errors in scanning from the reader
/// - An error produced by the underlying grep library during the search
pub fn scan_pattern(reader: &mut dyn Read, pattern: &str) -> Result<(), Error> {
	let matcher = RegexMatcher::new(pattern)?;
	let mut searcher = SearcherBuilder::new().passthru(true).build();
	let context_sink = sink::ContextPrintingSink::new();

	searcher.search_reader(matcher, reader, context_sink)?;
	Ok(())
}

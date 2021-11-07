#![warn(clippy::all, clippy::pedantic)]
use grep::regex::RegexMatcher;
use grep::searcher::SearcherBuilder;
use std::error::Error;
use std::io::Read;

mod print;
mod sink;

/// `scan_pattern` will print a reader's contents, while also scanning its contents for a regular expression.
/// Lines that match this pattern will be highlighted in the output.
///
/// Note that this pattern is not anchored at the start of the line by default, and therefore a match anywhere in the
/// line will force the entire line to be considered a match. For instance, the pattern `[a-z]` will match `123abc456`.
///
/// # Errors
///
/// There are two general error cases
/// - I/O errors in scanning from the reader
/// - An invalid regular expression.
// TODO: Find some way to differentiate between these error types
pub fn scan_pattern(reader: &mut dyn Read, pattern: &str) -> Result<(), Box<dyn Error>> {
	let matcher = RegexMatcher::new(pattern)?;
	let mut searcher = SearcherBuilder::new().passthru(true).build();
	let context_sink = sink::ContextPrintingSink::new();
	searcher.search_reader(matcher, reader, context_sink)
}

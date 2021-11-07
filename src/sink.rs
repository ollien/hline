use crate::print;
use crate::print::{Printer, StdoutPrinter};
use grep::searcher::{Searcher, Sink, SinkContext, SinkMatch};
use std::error;
use std::io;
use termion::color;
use thiserror::Error;

pub(crate) struct ContextPrintingSink<P: Printer> {
	printer: P,
}

#[derive(Error, Debug)]
enum Error {
	#[error("The given searcher does not have passthru enabled. This is required for proper functionality")]
	NoPassthruOnSearcher,
	#[error("Print failure: {0}")]
	PrintFailed(io::Error),
}

impl From<print::Error> for Error {
	fn from(err: print::Error) -> Self {
		let io_err = match err {
			print::Error::BrokenPipe(wrapped) | print::Error::Other(wrapped) => wrapped,
		};

		Error::PrintFailed(io_err)
	}
}

impl<P: Printer> ContextPrintingSink<P> {
	// TODO: this box should really just return an Error.
	fn get_sink_result_for_print_result(res: print::Result) -> Result<bool, Box<dyn error::Error>> {
		match res {
			// TODO: get rid of this boxing
			Err(print::Error::Other(_)) => Err(Box::new(Error::from(res.unwrap_err()))),
			// It is not an error case to have a broken pipe; it just means we can't output anything more and we
			// shouldn't keep searching
			Err(print::Error::BrokenPipe(_)) => Ok(false),
			Ok(_) => Ok(true),
		}
	}
}

impl ContextPrintingSink<StdoutPrinter> {
	pub fn new() -> Self {
		ContextPrintingSink::default()
	}
}

impl Default for ContextPrintingSink<StdoutPrinter> {
	fn default() -> Self {
		ContextPrintingSink {
			printer: StdoutPrinter {},
		}
	}
}

impl<P: Printer> ContextPrintingSink<P> {
	fn validate_searcher(searcher: &Searcher) -> Result<(), Error> {
		if !searcher.passthru() {
			return Err(Error::NoPassthruOnSearcher);
		}

		Ok(())
	}
}

impl<P: Printer> Sink for ContextPrintingSink<P> {
	// TODO: I may eventually reconfigure this to wrap this with an enum to differentiate improper configuration
	type Error = Box<dyn error::Error>;

	fn matched(
		&mut self,
		searcher: &Searcher,
		sink_match: &SinkMatch,
	) -> Result<bool, Self::Error> {
		Self::validate_searcher(searcher)?;

		let print_res = self.printer.colored_print(
			color::Fg(color::Red),
			std::str::from_utf8(sink_match.bytes()).unwrap(),
		);

		Self::get_sink_result_for_print_result(print_res)
	}

	fn context(
		&mut self,
		searcher: &Searcher,
		context: &SinkContext<'_>,
	) -> Result<bool, Self::Error> {
		Self::validate_searcher(searcher)?;

		let data = std::str::from_utf8(context.bytes()).unwrap();
		let print_res = self.printer.print(data);

		Self::get_sink_result_for_print_result(print_res)
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use grep::regex::RegexMatcher;
	use grep::searcher::SearcherBuilder;
	use std::cell::RefCell;
	use std::fmt;
	use std::io;
	use test_case::test_case;

	#[derive(Default)]
	struct MockPrinter {
		messages: RefCell<Vec<String>>,
		colored_messages: RefCell<Vec<String>>,
		next_error: RefCell<Option<print::Error>>,
	}

	impl MockPrinter {
		fn fail_next(&mut self, error: print::Error) {
			self.next_error.replace(Some(error));
		}
	}

	impl Printer for &MockPrinter {
		fn print<S: fmt::Display>(&self, msg: S) -> print::Result {
			self.messages.borrow_mut().push(msg.to_string());

			if self.next_error.borrow().is_some() {
				Err(self.next_error.replace(None).unwrap())
			} else {
				Ok(())
			}
		}

		fn colored_print<S: fmt::Display, C: color::Color>(
			&self,
			_color: color::Fg<C>,
			msg: S,
		) -> print::Result {
			// Unfortunately, termion colors don't implement PartialEq, so checking for the exact color is not
			// feasible unless we wanted to write a wrapper, which I don't care enough to just for unit testing
			self.colored_messages.borrow_mut().push(msg.to_string());

			if self.next_error.borrow().is_some() {
				Err(self.next_error.replace(None).unwrap())
			} else {
				Ok(())
			}
		}
	}

	const LIPSUM: &str = "Lorem ipsum dolor sit amet, consectetur adipiscing elit. Etiam vel magna a magna porta \n\
	viverra eu ac metus. Integer auctor enim id turpis mollis, quis sagittis nulla accumsan. Nam sagittis lorem ut \n\
	elit convallis ultricies. Suspendisse sed lobortis enim. Nulla lobortis tristique maximus. Mauris fermentum urna \n\
	id ex finibus commodo. Aliquam erat volutpat. Maecenas tristique erat vel consectetur varius. Fusce a \n\
	condimentum orci. Praesent at rhoncus felis, et tempus nulla. Morbi consectetur, elit quis interdum tincidunt, \n\
	felis risus malesuada elit, non feugiat tortor velit vel risus. Nullam a odio sodales, iaculis quam sit amet, \n\
	molestie dolor.  Praesent et nibh id nisl convallis hendrerit ac sed sapien. Fusce tempus venenatis odio, \n\
	ut maximus nisi egestas vel.";

	fn are_slices_eq<T: PartialEq>(v1: &[T], v2: &[T]) -> bool {
		if v1.len() != v2.len() {
			return false;
		}

		// https://stackoverflow.com/a/29504547
		let len = v1.len();
		v1.iter().zip(v2).filter(|&(a, b)| a == b).count() == len
	}

	#[test]
	fn test_highlights_matches() {
		let matcher = RegexMatcher::new("Integer").expect("regexp doesn't compile");

		let mock_printer = MockPrinter::default();
		let sink = ContextPrintingSink {
			printer: &mock_printer,
		};

		let res = SearcherBuilder::new().passthru(true).build().search_slice(
			matcher,
			LIPSUM.as_bytes(),
			sink,
		);
		if let Err(err) = res {
			panic!("failed to search: {}", err)
		}

		let colored_messages = mock_printer.colored_messages.borrow();
		let expected_colored_messages = ["viverra eu ac metus. Integer auctor enim id turpis mollis, quis sagittis nulla accumsan. Nam sagittis lorem ut \n".to_string()];
		assert!(
			are_slices_eq(&colored_messages, &expected_colored_messages),
			"(expected) {:?} != (actual) {:?}",
			expected_colored_messages,
			colored_messages,
		);

		let uncolored_messages = mock_printer.messages.borrow();
		let expected_uncolored_messages = [
			"Lorem ipsum dolor sit amet, consectetur adipiscing elit. Etiam vel magna a magna porta \n".to_string(),
			// only missing the line that's in expected
			"elit convallis ultricies. Suspendisse sed lobortis enim. Nulla lobortis tristique maximus. Mauris fermentum urna \n".to_string(),
			"id ex finibus commodo. Aliquam erat volutpat. Maecenas tristique erat vel consectetur varius. Fusce a \n".to_string(),
			"condimentum orci. Praesent at rhoncus felis, et tempus nulla. Morbi consectetur, elit quis interdum tincidunt, \n".to_string(),
			"felis risus malesuada elit, non feugiat tortor velit vel risus. Nullam a odio sodales, iaculis quam sit amet, \n".to_string(),
			"molestie dolor.  Praesent et nibh id nisl convallis hendrerit ac sed sapien. Fusce tempus venenatis odio, \n".to_string(),
			"ut maximus nisi egestas vel.".to_string()
		];
		assert!(
			are_slices_eq(&uncolored_messages, &expected_uncolored_messages),
			"(expected) {:?} != (actual) {:?}",
			expected_uncolored_messages,
			colored_messages,
		);
	}

	#[test_case(".", 0, 1; "failure on first match will only attempt to print that match")]
	#[test_case("this doesn't appear in lorem ipsum", 1, 0; "never matching will only attempt to print the first line")]
	fn test_does_not_attempt_to_print_after_broken_pipe_error(
		pattern: &str,
		num_uncolored_messages: usize,
		num_colored_messages: usize,
	) {
		let matcher = RegexMatcher::new(pattern).expect("regexp doesn't compile");

		let mut mock_printer = MockPrinter::default();

		let broken_pipe_err =
			print::Error::from(io::Error::new(io::ErrorKind::BrokenPipe, "broken pipe"));
		mock_printer.fail_next(broken_pipe_err);

		let sink = ContextPrintingSink {
			printer: &mock_printer,
		};

		let res = SearcherBuilder::new().passthru(true).build().search_slice(
			matcher,
			LIPSUM.as_bytes(),
			sink,
		);

		assert!(!res.is_err(), "failed to search: {:?}", res.unwrap_err());
		assert_eq!(
			num_colored_messages,
			mock_printer.colored_messages.borrow().len()
		);
		assert_eq!(num_uncolored_messages, mock_printer.messages.borrow().len());
	}

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
		let matcher = RegexMatcher::new("Integer").expect("regexp doesn't compile");

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
		let search_res = searcher.search_slice(matcher, LIPSUM.as_bytes(), sink);

		if valid {
			assert!(search_res.is_ok());
		} else {
			assert!(search_res.is_err());
		}
	}
}

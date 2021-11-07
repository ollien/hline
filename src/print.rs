use std::fmt;
use std::io;
use std::io::Write;
use std::result;
use termion::color::{Color, Fg};
use thiserror::Error;

pub(crate) type Result = result::Result<(), Error>;

/// Error is a simple wrapper for `io::Error` that differentiates between certain error kinds as part of the type.
///
/// In general, this type exists because of <https://github.com/rust-lang/rust/issues/46016>; println! panics on
/// broken pipe errors. Though we could just ignore the errors, we need some way to differentiate between it and other
/// errors. This could be done with `io::Error::kind`, but this wrapper makes it explicit it should be handled with an
/// action such as terminating gracefully.  It's silly and annoying, but it's how it is.
#[derive(Error, Debug)]
pub(crate) enum Error {
	#[error("{0}")]
	BrokenPipe(io::Error),
	#[error("{0}")]
	Other(io::Error),
}

impl From<io::Error> for Error {
	fn from(err: io::Error) -> Self {
		match err.kind() {
			io::ErrorKind::BrokenPipe => Self::BrokenPipe(err),
			_ => Self::Other(err),
		}
	}
}

/// Printer represents an object that can perform some kind of printing, such as by the print! macro
pub(crate) trait Printer {
	/// Print the given message.
	///
	/// # Errors
	/// In the event of any i/o error, an error is returned. The type [Error] gives implementors the freendom to specify
	/// whether or not this error was due to some kind of broken pipe error, which callers may choose to execute specifc
	/// behavior. The docs of [Error] specify more information about this.
	fn print<S: fmt::Display>(&self, msg: S) -> Result;

	/// Print the given message with the given foreground color.
	///
	/// # Errors
	/// In the event of any i/o error, an error is returned. The type [Error] gives implementors the freendom to specify
	/// whether or not this error was due to some kind of broken pipe error, which callers may choose to execute specifc
	/// behavior. The docs of [Error] specify more information about this.
	fn colored_print<S: fmt::Display, C: Color>(&self, color: Fg<C>, msg: S) -> Result {
		let colored_message = format!("{}{}", color, msg);

		self.print(colored_message)
	}
}

pub(crate) struct StdoutPrinter;

impl Printer for StdoutPrinter {
	fn print<S: fmt::Display>(&self, msg: S) -> Result {
		let mut stdout = io::stdout();
		Ok(write!(stdout, "{}", msg)?)
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use test_case::test_case;

	#[test_case(
		io::Error::new(io::ErrorKind::BrokenPipe, "broken pipe"),
		&|err| matches!(err, Error::BrokenPipe(_));
		"BrokenPipe results in BrokenPipe variant"
	)]
	#[test_case(
		io::Error::new(io::ErrorKind::Interrupted, "can't print, we're busy"),
		&|err| matches!(err, Error::Other(_));
		"non-BrokenPipe produces Other variant"
	)]
	fn test_error_from_io_err(from: io::Error, matches: &dyn Fn(&Error) -> bool) {
		let produced_err = Error::from(from);
		assert!(
			matches(&produced_err),
			"enum did not match: got {:?}",
			&produced_err
		);
	}
}

//! `print` provides utilities to facilitate printing out search results.
use crate::lines;
use std::fmt;
use std::io;
use std::io::Write;
use std::result;
use termion::color::{Color, Fg, Reset};
use thiserror::Error;

pub(crate) type Result = result::Result<(), Error>;

/// Error is a simple wrapper for [`io::Error`] that differentiates between certain error kinds as part of the type.
///
/// In general, this type exists because of <https://github.com/rust-lang/rust/issues/46016>; println! panics on
/// broken pipe errors. Though we could just ignore the errors, we need some way to differentiate between it and other
/// errors. This could be done with `io::Error::kind`, but this wrapper makes it explicit it should be handled with an
/// action such as terminating gracefully.  It's silly and annoying, but it's how it is.
#[derive(Error, Debug)]
#[non_exhaustive]
pub enum Error {
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

/// `Printer` represents an object that can perform some kind of printing, such as by the print! macro
pub trait Printer {
    /// Print the given message.
    ///
    /// # Errors
    /// In the event of any i/o error, an error is returned. The type [enum@Error] gives implementors the freedom to
    /// specify whether or not this error was due to some kind of broken pipe error, which callers may choose to execute
    /// specific behavior. The docs of [enum@Error] specify more information about this.
    fn print<S: fmt::Display>(&self, msg: S) -> Result;

    /// Print the given message with the given foreground color.
    ///
    /// # Errors
    /// In the event of any i/o error, an error is returned. The type [enum@Error] gives implementors the freedom to
    /// specify whether or not this error was due to some kind of broken pipe error, which callers may choose to
    /// execute specific behavior. The docs of [enum@Error] specify more information about this.
    fn colored_print<S: fmt::Display, C: Color>(&self, color: Fg<C>, msg: S) -> Result {
        let msg_string = msg.to_string();
        let colored_msg: String = lines::line_split(&msg_string)
            .map(|(component, joining_newline)| {
                if component.is_empty() {
                    return joining_newline.unwrap_or_default().to_string();
                }

                format!(
                    "{color}{component}{reset}{joining_newline}",
                    color = color,
                    reset = Fg(Reset),
                    component = component,
                    joining_newline = joining_newline.unwrap_or_default()
                )
            })
            .collect();

        self.print(colored_msg)
    }
}

/// `StdoutPrinter` is, quite simply, a printer that will print to stdout.
pub struct StdoutPrinter;

impl StdoutPrinter {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }
}

impl Default for StdoutPrinter {
    fn default() -> Self {
        Self {}
    }
}

impl Printer for StdoutPrinter {
    fn print<S: fmt::Display>(&self, msg: S) -> Result {
        let mut stdout = io::stdout();
        Ok(write!(stdout, "{}", msg)?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testutil;
    use crate::testutil::mock_print::BarebonesMockPrinter;
    use termion::color::Magenta;
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

    #[test_case(
        "hello world".to_string(),
        format!("{0}hello world{1}", Fg(Magenta), Fg(Reset));
        "no-newline case ends with reset"
    )]
    #[test_case(
        "foo\nbar\n".to_string(),
        format!("{0}foo{1}\n{0}bar{1}\n", Fg(Magenta), Fg(Reset));
        "puts reset char before newlines"
    )]
    #[test_case(
        "hello\n\n\nworld".to_string(),
        format!("{0}hello{1}\n\n\n{0}world{1}", Fg(Magenta), Fg(Reset));
        "empty strings don't need colorization"
    )]
    fn test_resets_colors_properly(message: String, expected: String) {
        // We're using a mock here specifically so we can test the default implementation of colored_print
        let printer = BarebonesMockPrinter::default();
        let res = printer.colored_print(Fg(Magenta), message);
        assert!(res.is_ok(), "{}", res.unwrap_err());

        testutil::assert_slices_eq!(&[expected], &printer.messages.borrow());
    }
}

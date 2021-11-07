use std::fmt;
use termion::color::{Color, Fg};

pub(crate) trait Printer {
	fn print<S: fmt::Display>(&self, msg: S);

	fn colored_print<S: fmt::Display, C: Color>(&self, color: Fg<C>, msg: S) {
		let colored_message = format!("{}{}", color, msg);
		self.print(colored_message);
	}
}

pub(crate) struct StdoutPrinter;

impl Printer for StdoutPrinter {
	fn print<S: fmt::Display>(&self, msg: S) {
		print!("{}", msg);
	}
}

#![cfg(test)]
use crate::print;
use crate::print::Printer;
use std::cell::RefCell;
use std::fmt;
use termion::color;

#[derive(Default)]
pub(crate) struct MockPrinter {
    pub(crate) messages: RefCell<Vec<String>>,
    pub(crate) colored_messages: RefCell<Vec<String>>,
    next_error: RefCell<Option<print::Error>>,
}

impl MockPrinter {
    pub(crate) fn fail_next(&mut self, error: print::Error) {
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

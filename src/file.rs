//! File provides utilities for reading files for scanning.
//!
//! These types are not generally require for using the methods defined in the crate root, but can be useful to
//! ensure their output will be usable.
mod recorder;
pub mod utf8;

pub use recorder::ReadRecorder;

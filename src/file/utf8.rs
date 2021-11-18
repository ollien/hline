//! Provides utilities to determine the properties of a UTF-8 encoded file.

// Many things in this file take heavy inspiration from less. As such, I'm including their license here.
//
//                           Less License
//                           ------------

// Less
// Copyright (C) 1984-2018  Mark Nudelman

// Redistribution and use in source and binary forms, with or without
// modification, are permitted provided that the following conditions
// are met:
// 1. Redistributions of source code must retain the above copyright
//    notice, this list of conditions and the following disclaimer.
// 2. Redistributions in binary form must reproduce the above copyright
//    notice in the documentation and/or other materials provided with
//    the distribution.

// THIS SOFTWARE IS PROVIDED BY THE AUTHOR ``AS IS'' AND ANY
// EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT LIMITED TO, THE
// IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR A PARTICULAR
// PURPOSE ARE DISCLAIMED.  IN NO EVENT SHALL THE AUTHOR BE LIABLE
// FOR ANY DIRECT, INDIRECT, INCIDENTAL, SPECIAL, EXEMPLARY, OR
// CONSEQUENTIAL DAMAGES (INCLUDING, BUT NOT LIMITED TO, PROCUREMENT
// OF SUBSTITUTE GOODS OR SERVICES; LOSS OF USE, DATA, OR PROFITS; OR
// BUSINESS INTERRUPTION) HOWEVER CAUSED AND ON ANY THEORY OF LIABILITY,
// WHETHER IN CONTRACT, STRICT LIABILITY, OR TORT (INCLUDING NEGLIGENCE
// OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE OF THIS SOFTWARE, EVEN
// IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.

use std::io::{Error, Read};
use std::ops::Range;

const BINARY_CHAR_THRESHOLD: i8 = 5;
const BUFFER_CHECK_AMOUNT: usize = 255;

/// `is_file_likely_binary` check if a file is likely a binary file. This is useful to check if a file is likely
/// human-readable or not.
///
/// # Errors
///
/// An [`std::io::Error`] will be returned if there is an underlying problem reading from the given [`Read`]
//
// This mechanism is inspired heavily by `less`' implementation, which follows the same semantics (in utf-8 mode,
// at least).
// https://github.com/gwsw/less/blob/294976950f5dc2a6b3436b1d2df97034936552b9/filename.c#L480-L484
#[allow(clippy::module_name_repetitions)]
pub fn is_file_likely_binary<R: Read>(file: &mut R) -> Result<bool, Error> {
    let mut buf: [u8; BUFFER_CHECK_AMOUNT] = [0; BUFFER_CHECK_AMOUNT];
    let bytes_read = file.read(&mut buf)?;

    let num_binary_chars = String::from_utf8_lossy(&buf[..bytes_read])
        .chars()
        .filter(|&c| was_utf8_char_replaced(c) || is_binary_char(c))
        .count();

    Ok(num_binary_chars > BINARY_CHAR_THRESHOLD as usize)
}

/// `was_utf8_char_replaced` checks if the given char was replaced by [`String::from_utf8_lossy`], which indicates that
/// it was not utf-8 originally
fn was_utf8_char_replaced(c: char) -> bool {
    c == std::char::REPLACEMENT_CHARACTER
}

/// `is_binary_char` checks if the given character is a binary char in the UTF-8 charset
fn is_binary_char(c: char) -> bool {
    // this table is stolen quite directly from Less
    // https://github.com/gwsw/less/blob/294976950f5dc2a6b3436b1d2df97034936552b9/ubin.uni
    // Because of the use of surrogate chars in this table, we cannot use rust's char type :(
    #[allow(clippy::unreadable_literal)]
    #[rustfmt::skip]
    let binary_codepoint_ranges = [
        Range::<u32>{ start: 0x0000,   end: 0x0007 + 1},   /* Cc */
        Range::<u32>{ start: 0x000b,   end: 0x000b + 1},   /* Cc */
        Range::<u32>{ start: 0x000e,   end: 0x001f + 1},   /* Cc */
        Range::<u32>{ start: 0x007f,   end: 0x009f + 1},   /* Cc */
        Range::<u32>{ start: 0x2028,   end: 0x2028 + 1},   /* Zl */
        Range::<u32>{ start: 0x2029,   end: 0x2029 + 1},   /* Zp */
        Range::<u32>{ start: 0xd800,   end: 0xd800 + 1},   /* Cs */
        Range::<u32>{ start: 0xdb7f,   end: 0xdb80 + 1},   /* Cs */
        Range::<u32>{ start: 0xdbff,   end: 0xdc00 + 1},   /* Cs */
        Range::<u32>{ start: 0xdfff,   end: 0xdfff + 1},   /* Cs */
        Range::<u32>{ start: 0xe000,   end: 0xe000 + 1},   /* Co */
        Range::<u32>{ start: 0xf8ff,   end: 0xf8ff + 1},   /* Co */
        Range::<u32>{ start: 0xf0000,  end: 0xf0000 + 1},  /* Co */
        Range::<u32>{ start: 0xffffd,  end: 0xffffd + 1},  /* Co */
        Range::<u32>{ start: 0x100000, end: 0x100000 + 1}, /* Co */
        Range::<u32>{ start: 0x10fffd, end: 0x10fffd + 1}, /* Co */
    ];

    binary_codepoint_ranges
        .into_iter()
        .any(|range| range.contains(&(c as u32)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;
    use test_case::test_case;

    #[test_case(b"hello", false; "simple string is utf-8")]
    #[test_case(b"hello\xff\xffworld", false; "single non-utf-8 is ok")]
    #[test_case(b"hello\x00world", false; "single binary char is ok")]
    #[test_case(b"hello\x00\xffworld", false; "single non-utf-8 and binary is ok")]
    #[test_case(b"hello\xff\xffworld\xfa\xfb\xfc\xfd\xfe", true; "too many non-utf-8 is not ok")]
    #[test_case(b"hello\0\0\0\0\0\0world", true; "null terms are binary chars")]
    #[test_case(b"\x7f\x45\x4c\x46\x02\x01\x01\x00\x00 ", true; "elf header is binary")]
    fn test_is_file_likely_utf8(s: &[u8], is_utf8: bool) {
        let mut byte_reader = Cursor::new(s);
        assert_eq!(is_utf8, is_file_likely_binary(&mut byte_reader).unwrap());
    }
}

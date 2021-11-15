use std::io::{Error, Read};

const NOT_UTF_8_THRESHOLD: i8 = 5;
const BUFFER_CHECK_AMOUNT: usize = 255;

/// `is_file_likely_utf8` check if a file is likely filled entirely with UTF-8 characters.
/// This is useful to check if a file is likely human-readable or not.
///
/// # Errors
///
/// An [`io::Error`] will be returned if there is an underlying problem reading from the given [`Read`]
//
// This mechanism is inspired heavily by `less`' implementation, which follows the same semantics (in utf-8 mode,
// at least).
// https://github.com/gwsw/less/blob/294976950f5dc2a6b3436b1d2df97034936552b9/filename.c#L480-L484
pub fn is_file_likely_utf8<R: Read>(file: &mut R) -> Result<bool, Error> {
    let mut buf: [u8; BUFFER_CHECK_AMOUNT] = [0; BUFFER_CHECK_AMOUNT];
    let bytes_read = file.read(&mut buf)?;

    let num_non_utf8_chars = String::from_utf8_lossy(&buf[..bytes_read])
        .chars()
        .filter(|c| c == &std::char::REPLACEMENT_CHARACTER)
        .count();

    Ok(num_non_utf8_chars <= NOT_UTF_8_THRESHOLD as usize)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;
    use test_case::test_case;

    #[test_case(b"hello", true; "simple string is utf-8")]
    #[test_case(b"hello\xff\xffworld", true; "single non-utf-8 is ok")]
    #[test_case(b"hello\xff\xffworld\xfa\xfb\xfc\xfd\xfe", false; "too many non-utf-8 is not ok")]
    fn test_is_file_likely_utf8(s: &[u8], is_utf8: bool) {
        let mut byte_reader = Cursor::new(s);
        assert_eq!(is_utf8, is_file_likely_utf8(&mut byte_reader).unwrap());
    }
}

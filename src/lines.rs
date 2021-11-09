//! `lines` provides utilities for processing lines of strings

/// `line_split` is an extremely similar iterator to `str::lines`, but with one key difference: it provides the line
/// character type it split on (the second element in the returned tuple). This way, one can reconstruct the original
/// string when joining. If the line was not terminated by a newline (i.e. when it's at the end of a file), the second
/// tuple element will be None.
pub(crate) fn line_split<'a>(s: &'a str) -> impl Iterator<Item = (&str, Option<&str>)> + 'a {
    // We could probably make this more efficient, but it would involve mostly re-implementing `split`.
    // I did some poking around, and this method is generally called for split_components.len() <= 2, so I'm not
    // too worried
    let split_components: Vec<&str> = s.split('\n').collect();
    let num_split_components = split_components.len();

    split_components
        .into_iter()
        .enumerate()
        .map(move |(idx, component)| {
            if idx == num_split_components - 1 {
                // The last split component will never have a newline, as otherwise it would have a ""
                // element following it
                return (component, None);
            } else if component.is_empty() {
                // If there's an empty component that _isn't_ the last component, it's going to be followed by a newline
                // (an \r\n terminated line will be non-empty).
                return (component, Some("\n"));
            }

            let len = component.len();
            if component.as_bytes()[len - 1] == b'\r' {
                (&component[0..len - 1], Some("\r\n"))
            } else {
                (component, Some("\n"))
            }
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testutil;
    use test_case::test_case;

    #[test_case(
        "hello",
        &[("hello", None) as (&str, Option<&str>)];
        "no newlines"
    )]
    #[test_case(
        "hello\nworld",
        &[("hello", Some("\n")), ("world", None)];
        "splitting newline"
    )]
    #[test_case(
        "hello\nworld\n",
        &[("hello", Some("\n")), ("world", Some("\n")), ("", None)];
        "terminating newlines"
    )]
    #[test_case(
        "hello\nworld\r\n",
        &[("hello", Some("\n")), ("world", Some("\r\n")), ("", None)];
        "mixing newline types"
    )]
    #[test_case(
        "hello\n\n\nworld",
        &[("hello", Some("\n")), ("", Some("\n")), ("", Some("\n")), ("world", None)];
        "chained newlines"
    )]
    #[test_case(
        "hello\n\r\n\nworld",
        &[("hello", Some("\n")), ("", Some("\r\n")), ("", Some("\n")), ("world", None)];
        "chained, mixed newlines"
    )]
    #[test_case(
        "hello\rworld\r\nthere it is!\n",
        &[("hello\rworld", Some("\r\n")), ("there it is!", Some("\n")), ("", None)];
        "carriage return alone isn't significant"
    )]
    fn test_splits_on_newlines(s: &str, expected: &[(&str, Option<&str>)]) {
        let collected: Vec<(&str, Option<&str>)> = line_split(s).collect();
        testutil::assert_slices_eq!(&expected, &collected);
    }
}

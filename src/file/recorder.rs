use std::cmp;
use std::io::{Error, Read};

// Having main() here helps with readability with the types I have to declare. Sorry clippy
#[allow(clippy::needless_doctest_main)]
/// `ReadRecorder` is a wrapper for [`Read`] that can "record" past reads for replay. This is especially useful if the
/// underlying [`Read`] does not implement [`Seek`](`std::io::Seek`).
///
/// # Examples
///
/// ```
/// use hline::file::ReadRecorder;
/// use std::io::{Cursor, Read, Result, Seek, SeekFrom};
///
/// // Wraps Cursor and disables seeking to demonstrate no seeking takes place
/// struct NoSeekCursor<T>(Cursor<T>);
///
/// impl <T> NoSeekCursor<T> {
///     fn new(data: T) -> Self {
///         Self(Cursor::new(data))
///     }
/// }
///
/// impl <T: AsRef<[u8]>> Read for NoSeekCursor<T> {
///     fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
///         self.0.read(buf)
///     }
/// }
///
/// impl <T> Seek for NoSeekCursor<T> {
///     fn seek(&mut self, _: SeekFrom) -> Result<u64> {
///         panic!("This cursor doesn't support seeking!");
///     }
/// }
///
/// fn main() {
///     let cursor = NoSeekCursor::new("hello world!");
///     let mut recorder = ReadRecorder::new(cursor);
///
///     recorder.start_recording();
///     // Read the first six chars ("hello ")
///     recorder.read(&mut [0u8; 6])
///         .expect("this read should have succeeded!");
///     recorder.stop_recording();
///     recorder.rewind_to_start_of_recording();
///
///     // We can now read the full string, including the data we already read, without seeing!
///     let mut read_data = String::new();
///     recorder.read_to_string(&mut read_data)
///         .expect("this read should have succeeded!");
///
///     assert_eq!(read_data, "hello world!");
/// }
/// ```
#[allow(clippy::module_name_repetitions)]
pub struct ReadRecorder<R: Read> {
    read: R,
    recorded_data: Vec<u8>,
    cursor_pos: Option<usize>,
    recording: bool,
}

impl<R: Read> ReadRecorder<R> {
    /// Make a new `ReadRecorder` wrapping the given `Reader`.
    pub fn new(reader: R) -> Self {
        Self {
            read: reader,
            recorded_data: Vec::new(),
            cursor_pos: None,
            recording: false,
        }
    }

    /// `start_recording` begins the recording process. Once this is started, any reads that call to the underlying
    /// [`Read`] (i.e. not through the recorded portion) will be copied to an internal buffer.
    pub fn start_recording(&mut self) {
        self.recording = true;
    }

    /// `stop_recording` stops additional reads from being recorded.
    pub fn stop_recording(&mut self) {
        self.recording = false;
    }

    /// `rewind_to_start_of_recording` is conceptually similar to `[Seek::rewind]`, except it will rewind only to the
    /// start of the recorded data.
    pub fn rewind_to_start_of_recording(&mut self) {
        self.cursor_pos = Some(0);
    }

    /// `copy_from_recording` will copy as much data as possible from the current recorded data to the given buffer
    fn copy_from_recording(&mut self, buf: &mut [u8]) -> usize {
        if self.cursor_pos.is_none() {
            return 0;
        }

        let cursor_pos = self.cursor_pos.unwrap();
        if cursor_pos >= self.recorded_data.len() {
            return 0;
        }

        let bytes_remaining_in_recording = self.recorded_data.len() - cursor_pos;
        let bytes_to_read = cmp::min(buf.len(), bytes_remaining_in_recording);
        self.recorded_data
            .iter()
            .skip(cursor_pos)
            .take(bytes_to_read)
            .enumerate()
            .for_each(|(idx, &chr)| buf[idx] = chr);

        self.cursor_pos = Some(cursor_pos + bytes_to_read);
        bytes_to_read
    }

    fn cursor_out_of_recording_bounds(&self) -> bool {
        match self.cursor_pos {
            None => false,
            Some(cursor_pos) => cursor_pos >= self.recorded_data.len(),
        }
    }

    fn should_clear_recorded_data(&self, num_bytes_read_from_file: usize) -> bool {
        // This takes a bit of mental energy to reason about, but the core idea of why we _also_ check the number
        // of bytes read from the file is that it should be perfectly allowable to go to the end of the recording, and
        // then re-read.
        //
        // If we don't have this stipulation, we can simply read up to the end of the buffer, at which point our cursor
        // is now out of bounds. We need to "wait" until we've actually performed some kind of read to the file.
        num_bytes_read_from_file > 0 && self.cursor_out_of_recording_bounds()
    }

    fn drop_recorded_data(&mut self) {
        self.recorded_data.clear();
        self.recorded_data.shrink_to_fit();
        self.cursor_pos = None;
    }
}

impl<R: Read> Read for ReadRecorder<R> {
    /// `read` serves two purposes. In the general case, it will forward reads to the wrapped [`Read`], and, if
    /// recording currently taking place (see [`start_recording`](`ReadRecorder::start_recording`)),
    /// then the read data will be copied to an internal data buffer. However, if the following two conditions are met,
    /// then it will not read from the wrapped [`Read`], but rather from the recording buffer.
    ///
    ///  1. there is recorded data to read
    ///  2. the internal "rewind cursor" is within the bounds of the read data.
    ///
    /// This "rewind cursor" is initialized by calling
    /// [`rewind_to_start_of_recording`](`ReadRecorder::rewind_to_start_of_recording`), which sets it to zero. Every
    /// byte read will advance this cursor, until it is outside the bounds of the recorded data, at which point the
    /// recorded data is dropped.
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, Error> {
        let bytes_copied_from_recording = self.copy_from_recording(buf);
        let bytes_read_from_file = self.read.read(&mut buf[bytes_copied_from_recording..])?;
        if self.recording {
            self.recorded_data
                .extend(buf.iter().take(bytes_read_from_file));
        } else if self.should_clear_recorded_data(bytes_read_from_file) {
            self.drop_recorded_data();
        }

        Ok(bytes_copied_from_recording + bytes_read_from_file)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{Cursor, Error};

    // A small wrapper for Cursor to provide a read "mock"
    struct ReadCountingCursor<R> {
        wrapped_cursor: Cursor<R>,
        // We will define a read as the number of non-trivial reads: i.e, more than 0 bytes are read
        num_reads: i32,
    }

    impl<R> ReadCountingCursor<R> {
        fn new(cursor: Cursor<R>) -> Self {
            Self {
                wrapped_cursor: cursor,
                num_reads: 0,
            }
        }
    }

    impl<R: AsRef<[u8]>> Read for ReadCountingCursor<R> {
        fn read(&mut self, buf: &mut [u8]) -> Result<usize, Error> {
            // If we are called with a non-trivial buffer, we can count the read
            if !buf.is_empty() {
                self.num_reads += 1;
            }

            self.wrapped_cursor.read(buf)
        }
    }

    #[test]
    fn test_reads_transparently_by_default() {
        let s_reader = Cursor::new("hello world");
        let mut recorder = ReadRecorder::new(s_reader);

        let mut read_out = String::new();
        recorder
            .read_to_string(&mut read_out)
            .expect("reading failed unexpectedly");

        assert_eq!(read_out, "hello world");
    }

    #[test]
    fn test_can_read_recorded_portion_repeatedly_without_calling_underlying_reader() {
        let s_reader = ReadCountingCursor::new(Cursor::new("hello world"));

        let mut recorder = ReadRecorder::new(s_reader);
        recorder.start_recording();

        // Read through the full string once to initialize the recording
        recorder
            .read_to_string(&mut String::new())
            .expect("reading failed unexpectedly");

        recorder.stop_recording();
        recorder.rewind_to_start_of_recording();

        let num_reads_before_retrying = recorder.read.num_reads;
        for _ in 0..3 {
            let mut read_out = [0_u8; "hello world".len()];
            recorder
                .read_exact(&mut read_out)
                .expect("reading failed unexpectedly");

            assert_eq!(
                std::str::from_utf8(&read_out).expect("did not read utf-8"),
                "hello world",
                "read data did not match expected"
            );
            recorder.rewind_to_start_of_recording();
            assert_eq!(
                num_reads_before_retrying, recorder.read.num_reads,
                "underlying Read was called more times than it should"
            );
        }
    }

    #[test]
    fn can_read_slowly_through_recording() {
        let s_reader = Cursor::new("hello world");
        let mut recorder = ReadRecorder::new(s_reader);
        recorder.start_recording();
        recorder
            .read_to_string(&mut String::new())
            .expect("reading failed unexpectedly");
        recorder.stop_recording();
        recorder.rewind_to_start_of_recording();

        let mut full_res = String::new();
        let mut buf = [0_u8; 2];
        loop {
            let bytes_read = recorder
                .read(&mut buf)
                .expect("reading failed unexpectedly");
            if bytes_read == 0 {
                break;
            }

            for byte in &buf[0..bytes_read] {
                full_res.push(*byte as char);
            }
        }

        assert_eq!("hello world", full_res);
    }

    #[test]
    fn test_does_not_call_underlying_read_when_reading_within_recorded_portion() {
        let s_reader = ReadCountingCursor::new(Cursor::new("hello world"));
        let mut recorder = ReadRecorder::new(s_reader);

        recorder.start_recording();
        recorder
            .read_exact(&mut [0_u8; 3])
            .expect("reading failed unexpectedly");
        recorder.rewind_to_start_of_recording();
        recorder.stop_recording();

        let num_reads_before = recorder.read.num_reads;
        recorder
            .read_exact(&mut [0_u8; 3])
            .expect("reading failed unexpectedly");

        assert_eq!(
            num_reads_before, recorder.read.num_reads,
            "underlying Read was called more times than it should"
        );
    }

    #[test]
    fn test_can_rewind_through_zero_length_recording() {
        let s_reader = ReadCountingCursor::new(Cursor::new("hello world"));
        let mut recorder = ReadRecorder::new(s_reader);
        recorder.rewind_to_start_of_recording();

        let mut read_contents = String::new();
        recorder
            .read_to_string(&mut read_contents)
            .expect("reading failed unexpectedly");

        assert_eq!(read_contents, "hello world");
    }

    #[test]
    fn test_reading_past_recorded_portion_drops_recording() {
        const BYTES_TO_RECORD: usize = 3;
        let s_reader = Cursor::new("hello world");
        let mut recorder = ReadRecorder::new(s_reader);

        recorder.start_recording();
        recorder
            .read_exact(&mut [0_u8; BYTES_TO_RECORD])
            .expect("reading failed unexpectedly");
        recorder.rewind_to_start_of_recording();
        recorder.stop_recording();

        // Read one byte past the recording
        recorder
            .read_exact(&mut [0_u8; BYTES_TO_RECORD + 1])
            .expect("reading failed unexpectedly");

        recorder.rewind_to_start_of_recording();

        // We should not be able to simply read the recorded data now. The recording buffer will be empty
        let mut read_contents = [0_u8; BYTES_TO_RECORD];
        recorder
            .read_exact(&mut read_contents)
            .expect("reading failed unexpectedly");

        assert_ne!(
            std::str::from_utf8(&read_contents).expect("did not read utf-8"),
            "hel",
            "Read data that the read cursor should have already passed"
        );
    }
}

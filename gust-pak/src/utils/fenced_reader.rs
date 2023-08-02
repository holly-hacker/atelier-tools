use std::io::{Read, Seek, SeekFrom};

/// A Read/Seek wrapper that only allows and seeking reading within a certain range.
///
/// This is an alternative to [std::io::Read::take], which does not support seeking.
pub struct FencedReader<R: Read + Seek> {
	inner: R,
	start: u64,
	end: u64,
	current: u64,
}

impl<R: Read + Seek> FencedReader<R> {
	/// Create a new instance of `FencedReader` that starts at the given reader's current position
	/// and ends at the given length.
	pub fn take(mut inner: R, len: u64) -> std::io::Result<Self> {
		let current = inner.stream_position()?;
		let start = current;
		let end = start + len;

		// ensure that `end` is still within bounds
		inner.seek(SeekFrom::Start(end))?;
		inner.seek(SeekFrom::Start(current))?;

		assert!(start <= end, "start must be <= end");
		Ok(Self {
			inner,
			start,
			end,
			current,
		})
	}
}

impl<R: Read + Seek> Read for FencedReader<R> {
	fn read(&mut self, mut buf: &mut [u8]) -> std::io::Result<usize> {
		debug_assert!(self.current <= self.end, "read past end of fence");

		let allowed_to_read = (self.end - self.current) as usize;
		if allowed_to_read < buf.len() {
			buf = &mut buf[..allowed_to_read];
		}

		let read = self.inner.read(buf)?;
		self.current += read as u64;
		Ok(read)
	}
}

impl<R: Read + Seek> Seek for FencedReader<R> {
	fn seek(&mut self, pos: SeekFrom) -> std::io::Result<u64> {
		let new_pos = match pos {
			SeekFrom::Start(offset) => self.start + offset,
			SeekFrom::End(offset) => (self.end as i64 + offset) as u64,
			SeekFrom::Current(offset) => (self.current as i64 + offset) as u64,
		};

		if !(self.start..=self.end).contains(&new_pos) {
			return Err(std::io::Error::new(
				std::io::ErrorKind::InvalidInput,
				"seek out of bounds",
			));
		}

		self.current = new_pos;
		let seek_to = self.inner.seek(SeekFrom::Start(new_pos))?;
		Ok(seek_to - self.start)
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	/// Returns a buffer to a slice of a [std::io::Cursor].
	fn get_buffer() -> FencedReader<std::io::Cursor<&'static [u8; 17]>> {
		let mut cursor = std::io::Cursor::new(b"__Hello, world!__");
		cursor.seek(SeekFrom::Start(2)).unwrap();
		FencedReader::take(cursor, b"Hello, world!".len() as u64).unwrap()
	}

	#[test]
	fn read_entire_buffer() {
		let mut reader = get_buffer();
		let mut buf = [0; 13];
		reader.read_exact(&mut buf).unwrap();
		assert_eq!(&buf, b"Hello, world!");
	}

	#[test]
	fn read_past_end() {
		let mut reader = get_buffer();
		let mut buf = [0; 14];
		let result = reader.read_exact(&mut buf);
		assert!(result.is_err());
	}

	#[test]
	fn seek_to_end_and_read() {
		let mut reader = get_buffer();
		reader.seek(SeekFrom::End(0)).unwrap();
		let mut buf = [0; 1];
		let result = reader.read_exact(&mut buf);
		assert!(result.is_err());
	}

	#[test]
	fn seek_inside_buffer_forwards_from_start() {
		let mut reader = get_buffer();
		reader.seek(SeekFrom::Start(7)).unwrap();
		let mut buf = [0; 6];
		reader.read_exact(&mut buf).unwrap();
		assert_eq!(&buf, b"world!");
	}

	#[test]
	fn seek_inside_buffer_backwards_from_end() {
		let mut reader = get_buffer();
		reader.seek(SeekFrom::End(-6)).unwrap();
		let mut buf = [0; 6];
		reader.read_exact(&mut buf).unwrap();
		assert_eq!(&buf, b"world!");
	}

	#[test]
	fn seek_inside_buffer_forwards_from_current() {
		let mut reader = get_buffer();
		reader.seek(SeekFrom::Current(7)).unwrap();
		let mut buf = [0; 6];
		reader.read_exact(&mut buf).unwrap();
		assert_eq!(&buf, b"world!");
	}

	#[test]
	fn seek_inside_buffer_backwards_from_current() {
		let mut reader = get_buffer();
		reader.seek(SeekFrom::End(0)).unwrap();
		reader.seek(SeekFrom::Current(-6)).unwrap();
		let mut buf = [0; 6];
		reader.read_exact(&mut buf).unwrap();
		assert_eq!(&buf, b"world!");
	}

	#[test]
	fn seek_outside_buffer_forwards_from_start() {
		let mut reader = get_buffer();
		let result = reader.seek(SeekFrom::Start(14));
		assert!(result.is_err());
	}

	#[test]
	fn seek_outside_buffer_backwards_from_end() {
		let mut reader = get_buffer();
		let result = reader.seek(SeekFrom::End(-14));
		assert!(result.is_err());
	}

	#[test]
	fn seek_outside_buffer_forwards_from_current() {
		let mut reader = get_buffer();
		let result = reader.seek(SeekFrom::Current(14));
		assert!(result.is_err());
	}

	#[test]
	fn seek_outside_buffer_backwards_from_current() {
		let mut reader = get_buffer();
		let result = reader.seek(SeekFrom::Current(-14));
		assert!(result.is_err());
	}

	#[test]
	fn seek_response_is_correct() {
		let mut reader = get_buffer();

		assert_eq!(
			reader.seek(SeekFrom::Start(0)).unwrap(),
			0,
			"seeking to start must return 0"
		);
		assert_eq!(reader.seek(SeekFrom::End(0)).unwrap(), 13);
	}
}

use std::io::{Read, Seek};

/// A reader that XORs the data with a key up to 32 bytes in length.
///
/// It is important to note that it does not support seeking, it will only read from the reader and
/// xor the data. It will however keep track of the current offset to ensure the correct position in
/// the xor key is used.
pub struct XorReader<R: Read + Seek> {
	inner: R,
	key_buffer: [u8; 32],
	key_len: usize,
}

impl<R: Read + Seek> XorReader<R> {
	/// Create a new instance of `XorReader` with the given key.
	pub fn new(inner: R, key_slice: &[u8]) -> Self {
		assert!(key_slice.len() <= 32);
		let mut key_buffer = [0u8; 32];
		key_buffer[..key_slice.len()].copy_from_slice(key_slice);
		Self {
			inner,
			key_buffer,
			key_len: key_slice.len(),
		}
	}
}

impl<R: Read + Seek> Read for XorReader<R> {
	fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
		let current_offset = self.inner.stream_position()?;

		let read = self.inner.read(buf)?;

		// decrypt read buffer
		buf.iter_mut().take(read).enumerate().for_each(|(i, b)| {
			*b ^= self.key_buffer[(current_offset as usize + i) % self.key_len];
		});

		Ok(read)
	}
}

impl<R: Read + Seek> Seek for XorReader<R> {
	fn seek(&mut self, pos: std::io::SeekFrom) -> std::io::Result<u64> {
		self.inner.seek(pos)
	}
}

#[cfg(test)]
mod tests {
	use std::io::SeekFrom;

	use super::*;

	#[test]
	fn read_text() {
		let mut reader = XorReader::new(
			std::io::Cursor::new(b"Hello, world!"),
			&[0x12, 0x34, 0x56, 0x78],
		);
		let mut buf = [0; 13];
		reader.read_exact(&mut buf).unwrap();
		assert_eq!(
			buf,
			[0x5a, 0x51, 0x3a, 0x14, 0x7d, 0x18, 0x76, 0x0f, 0x7d, 0x46, 0x3a, 0x1c, 0x33]
		);
	}

	/// Tests that the reader can correctly `read` chunks that are not divisible by the key length.
	#[test]
	fn read_text_unaligned_chunks() {
		let mut reader = XorReader::new(
			std::io::Cursor::new(b"Hello, world!"),
			&[0x12, 0x34, 0x56, 0x78],
		);

		let mut buf = [0; 13];
		assert_eq!(reader.read(&mut buf[0..3]).unwrap(), 3);
		assert_eq!(reader.read(&mut buf[3..8]).unwrap(), 5);
		assert_eq!(reader.read(&mut buf[8..10]).unwrap(), 2);
		assert_eq!(reader.read(&mut buf[10..13]).unwrap(), 3);

		assert_eq!(
			buf,
			[0x5a, 0x51, 0x3a, 0x14, 0x7d, 0x18, 0x76, 0x0f, 0x7d, 0x46, 0x3a, 0x1c, 0x33]
		);
	}

	/// Tests that the reader can correctly `seek` around and still correctly decrypt the data.
	#[test]
	fn read_text_unaligned_chunks_seek() {
		let mut reader = XorReader::new(
			std::io::Cursor::new(b"Hello, world!"),
			&[0x12, 0x34, 0x56, 0x78],
		);

		let mut buf = [0; 13];
		let sought_to = reader.seek(SeekFrom::Start(3)).unwrap();
		reader.read_exact(&mut buf[3..8]).unwrap();
		assert_eq!(sought_to, 3);

		let sought_to = reader.seek(SeekFrom::End(-3)).unwrap();
		reader.read_exact(&mut buf[10..13]).unwrap();
		assert_eq!(sought_to, 10);

		let sought_to = reader.seek(SeekFrom::Current(-13)).unwrap();
		reader.read_exact(&mut buf[0..3]).unwrap();
		assert_eq!(sought_to, 0);

		let sought_to = reader.seek(SeekFrom::Current(5)).unwrap();
		reader.read_exact(&mut buf[8..10]).unwrap();
		assert_eq!(sought_to, 8);

		assert_eq!(
			buf,
			[0x5a, 0x51, 0x3a, 0x14, 0x7d, 0x18, 0x76, 0x0f, 0x7d, 0x46, 0x3a, 0x1c, 0x33]
		);
	}

	/// Ensure a reader that doesn't start at position 0 is still correctly decrypted
	#[test]
	fn read_text_unaligned_chunks_nonzero_start() {
		let mut cursor = std::io::Cursor::new(b"Hello, world!");
		cursor.seek(SeekFrom::Start(6)).unwrap();

		let mut reader = XorReader::new(cursor, &[0x12, 0x34, 0x56, 0x78]);
		reader.seek(SeekFrom::Start(0)).unwrap();

		let mut buf = [0; 13];
		reader.read_exact(&mut buf).unwrap();

		assert_eq!(
			buf,
			[0x5a, 0x51, 0x3a, 0x14, 0x7d, 0x18, 0x76, 0x0f, 0x7d, 0x46, 0x3a, 0x1c, 0x33]
		);
	}
}

use std::io::{Seek, SeekFrom};

/// `Seek::stream_len` taken from the standard library, where this function is currently unstable.
///
/// See: [rust-lang/rust#59359](https://github.com/rust-lang/rust/issues/59359)
pub fn stream_len<T: Seek>(stream: &mut T) -> std::io::Result<u64> {
	let old_pos = stream.stream_position()?;
	let len = stream.seek(SeekFrom::End(0))?;

	// Avoid seeking a third time when we were already at the end of the
	// stream. The branch is usually way cheaper than a seek operation.
	if old_pos != len {
		stream.seek(SeekFrom::Start(old_pos))?;
	}

	Ok(len)
}

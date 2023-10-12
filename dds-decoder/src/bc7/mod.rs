mod blocks;

use crate::{errors::Bc7Error, Color4};

pub fn read_image(data: &[u8], width: usize, height: usize) -> Result<Vec<u8>, Bc7Error> {
	let blocks_x = usize::max(1, (width + 3) / 4);
	let blocks_y = usize::max(1, (height + 3) / 4);
	let block_count = blocks_x * blocks_y;
	let decoded_pixel_count = block_count * 16;

	// assume 128 bits are read at once (16 bytes)
	// the size of each chunk is a 4x4 block of rgba8 pixels
	let mut decoded_pixels = vec![Color4::default(); decoded_pixel_count];
	for (chunk_index, chunk) in data.chunks_exact(16).enumerate() {
		let span = tracing::trace_span!("chunk", chunk_index);
		let _guard = span.enter();

		let chunk = blocks::decode(chunk)?;

		// copy the chunk into the decoded pixels
		let chunk_x = (chunk_index % blocks_x) * 4;
		let chunk_y = (chunk_index / blocks_x) * 4;
		let target_index = chunk_y * (blocks_x * 4) + chunk_x;

		#[allow(clippy::needless_range_loop)]
		for y in 0..4 {
			for x in 0..4 {
				let target_index = target_index + y * (blocks_x * 4) + x;
				decoded_pixels[target_index] = chunk[y][x];
			}
		}
	}

	let decoded_pixels = decoded_pixels
		.into_iter()
		.flat_map(|color| color.components)
		.collect::<Vec<_>>();

	// the decoded pixels may contain some "padding" on each row, since the width and height may not
	// be divisible by the decoded block size
	let mut final_pixels = vec![0u8; width * height * 4];
	let decoded_pixels_line_bytes = blocks_x * 4 * 4;
	decoded_pixels
		.chunks_exact(decoded_pixels_line_bytes)
		.take(height)
		.enumerate()
		.for_each(|(row, line)| {
			let line_offset = width * row * 4;
			final_pixels[line_offset..(line_offset + width * 4)]
				.copy_from_slice(&line[0..width * 4]);
		});

	tracing::debug!("image decoded");

	Ok(final_pixels)
}

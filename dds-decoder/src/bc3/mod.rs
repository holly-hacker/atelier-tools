#![allow(clippy::needless_range_loop)]

// This code is mostly based on the official microsoft documentation:
// https://learn.microsoft.com/en-us/windows/win32/direct3d9/textures-with-alpha-channels#three-bit-linear-alpha-interpolation

use crate::{
	util::{interp_color_2_opaque, interp_color_3_opaque, unpack_dxt_color_565},
	Color4, ColorBlock,
};

pub fn read_image(data: &[u8], width: usize, height: usize) -> Vec<u8> {
	let blocks_x = usize::max(1, (width + 3) / 4);
	let blocks_y = usize::max(1, (height + 3) / 4);
	let block_count = blocks_x * blocks_y;
	let decoded_pixel_count = block_count * 16;

	// BC3 has a chunk size of 8 bytes which decodes in a 4x4 block of pixels (64 bytes as RGBA8)
	let mut decoded_pixels = vec![Color4::default(); decoded_pixel_count];
	for (chunk_index, chunk) in data.chunks_exact(16).enumerate() {
		let chunk = decode_block(chunk);

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

	final_pixels
}

fn decode_block(block: &[u8]) -> ColorBlock {
	debug_assert_eq!(block.len(), 16);

	let alpha_0 = block[0] as usize;
	let alpha_1 = block[1] as usize;

	// 3bpp 4x4 bitmap
	let alpha_bitmap = u64::from_le_bytes([
		block[2], block[3], block[4], block[5], block[6], block[7], 0, 0,
	]);

	let color_0 = unpack_dxt_color_565(u16::from_le_bytes([block[8], block[9]]));
	let color_1 = unpack_dxt_color_565(u16::from_le_bytes([block[10], block[11]]));

	let color_bitmap = u32::from_le_bytes([block[12], block[13], block[14], block[15]]);

	#[allow(clippy::identity_op)]
	let alpha: [u8; 8] = if alpha_0 > alpha_1 {
		[
			alpha_0 as u8,
			alpha_1 as u8,
			// six intermediate alpha values created by interpolation between the two endpoints
			((6 * alpha_0 + 1 * alpha_1 + 3) / 7) as u8,
			((5 * alpha_0 + 2 * alpha_1 + 3) / 7) as u8,
			((4 * alpha_0 + 3 * alpha_1 + 3) / 7) as u8,
			((3 * alpha_0 + 4 * alpha_1 + 3) / 7) as u8,
			((2 * alpha_0 + 5 * alpha_1 + 3) / 7) as u8,
			((1 * alpha_0 + 6 * alpha_1 + 3) / 7) as u8,
		]
	} else {
		[
			alpha_0 as u8,
			alpha_1 as u8,
			// four intermediate alpha values created by interpolation between the two endpoints
			((4 * alpha_0 + 1 * alpha_1 + 2) / 5) as u8,
			((3 * alpha_0 + 2 * alpha_1 + 2) / 5) as u8,
			((2 * alpha_0 + 3 * alpha_1 + 2) / 5) as u8,
			((1 * alpha_0 + 4 * alpha_1 + 2) / 5) as u8,
			// the last two alpha values are the min and max alpha value
			0,
			255,
		]
	};

	let mut color_block = ColorBlock::default();
	for y in 0..4 {
		for x in 0..4 {
			let alpha_bits = (alpha_bitmap >> ((x + y * 4) * 3)) & 0b111;
			let alpha = alpha[alpha_bits as usize];

			let color_0 = color_0.with_alpha(alpha);
			let color_1 = color_1.with_alpha(alpha);

			let bits = (color_bitmap >> ((x + y * 4) * 2)) & 0b11;
			color_block[y][x] = match bits {
				0b00 => color_0,
				0b01 => color_1,
				0b10 => interp_color_2_opaque(color_0, color_1),
				0b11 => interp_color_3_opaque(color_0, color_1),
				_ => unreachable!(),
			};
		}
	}

	color_block
}

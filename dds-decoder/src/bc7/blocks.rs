#![allow(clippy::needless_range_loop)]

// based on bc7enc: https://github.com/richgel999/bc7enc/blob/f66c2e489b07138f2673a2fb3d27c1aa1d565c48/bc7decomp.cpp
// bc7enc is released in the public domain

// TODO: implement correct error handling for this module
// TODO: consider using const generics to make the `mode` parameter generic. may help with speed.

use crate::{errors::Bc7Error, Color4};

type ColorBlock = [[Color4; 4]; 4];

#[rustfmt::skip]
const PARTITION2: [u8; 64 * 16] =
[
	0,0,1,1,0,0,1,1,0,0,1,1,0,0,1,1, 0,0,0,1,0,0,0,1,0,0,0,1,0,0,0,1, 0,1,1,1,0,1,1,1,0,1,1,1,0,1,1,1, 0,0,0,1,0,0,1,1,0,0,1,1,0,1,1,1, 0,0,0,0,0,0,0,1,0,0,0,1,0,0,1,1, 0,0,1,1,0,1,1,1,0,1,1,1,1,1,1,1, 0,0,0,1,0,0,1,1,0,1,1,1,1,1,1,1, 0,0,0,0,0,0,0,1,0,0,1,1,0,1,1,1,
	0,0,0,0,0,0,0,0,0,0,0,1,0,0,1,1, 0,0,1,1,0,1,1,1,1,1,1,1,1,1,1,1, 0,0,0,0,0,0,0,1,0,1,1,1,1,1,1,1, 0,0,0,0,0,0,0,0,0,0,0,1,0,1,1,1, 0,0,0,1,0,1,1,1,1,1,1,1,1,1,1,1, 0,0,0,0,0,0,0,0,1,1,1,1,1,1,1,1, 0,0,0,0,1,1,1,1,1,1,1,1,1,1,1,1, 0,0,0,0,0,0,0,0,0,0,0,0,1,1,1,1,
	0,0,0,0,1,0,0,0,1,1,1,0,1,1,1,1, 0,1,1,1,0,0,0,1,0,0,0,0,0,0,0,0, 0,0,0,0,0,0,0,0,1,0,0,0,1,1,1,0, 0,1,1,1,0,0,1,1,0,0,0,1,0,0,0,0, 0,0,1,1,0,0,0,1,0,0,0,0,0,0,0,0, 0,0,0,0,1,0,0,0,1,1,0,0,1,1,1,0, 0,0,0,0,0,0,0,0,1,0,0,0,1,1,0,0, 0,1,1,1,0,0,1,1,0,0,1,1,0,0,0,1,
	0,0,1,1,0,0,0,1,0,0,0,1,0,0,0,0, 0,0,0,0,1,0,0,0,1,0,0,0,1,1,0,0, 0,1,1,0,0,1,1,0,0,1,1,0,0,1,1,0, 0,0,1,1,0,1,1,0,0,1,1,0,1,1,0,0, 0,0,0,1,0,1,1,1,1,1,1,0,1,0,0,0, 0,0,0,0,1,1,1,1,1,1,1,1,0,0,0,0, 0,1,1,1,0,0,0,1,1,0,0,0,1,1,1,0, 0,0,1,1,1,0,0,1,1,0,0,1,1,1,0,0,
	0,1,0,1,0,1,0,1,0,1,0,1,0,1,0,1, 0,0,0,0,1,1,1,1,0,0,0,0,1,1,1,1, 0,1,0,1,1,0,1,0,0,1,0,1,1,0,1,0, 0,0,1,1,0,0,1,1,1,1,0,0,1,1,0,0, 0,0,1,1,1,1,0,0,0,0,1,1,1,1,0,0, 0,1,0,1,0,1,0,1,1,0,1,0,1,0,1,0, 0,1,1,0,1,0,0,1,0,1,1,0,1,0,0,1, 0,1,0,1,1,0,1,0,1,0,1,0,0,1,0,1,
	0,1,1,1,0,0,1,1,1,1,0,0,1,1,1,0, 0,0,0,1,0,0,1,1,1,1,0,0,1,0,0,0, 0,0,1,1,0,0,1,0,0,1,0,0,1,1,0,0, 0,0,1,1,1,0,1,1,1,1,0,1,1,1,0,0, 0,1,1,0,1,0,0,1,1,0,0,1,0,1,1,0, 0,0,1,1,1,1,0,0,1,1,0,0,0,0,1,1, 0,1,1,0,0,1,1,0,1,0,0,1,1,0,0,1, 0,0,0,0,0,1,1,0,0,1,1,0,0,0,0,0,
	0,1,0,0,1,1,1,0,0,1,0,0,0,0,0,0, 0,0,1,0,0,1,1,1,0,0,1,0,0,0,0,0, 0,0,0,0,0,0,1,0,0,1,1,1,0,0,1,0, 0,0,0,0,0,1,0,0,1,1,1,0,0,1,0,0, 0,1,1,0,1,1,0,0,1,0,0,1,0,0,1,1, 0,0,1,1,0,1,1,0,1,1,0,0,1,0,0,1, 0,1,1,0,0,0,1,1,1,0,0,1,1,1,0,0, 0,0,1,1,1,0,0,1,1,1,0,0,0,1,1,0,
	0,1,1,0,1,1,0,0,1,1,0,0,1,0,0,1, 0,1,1,0,0,0,1,1,0,0,1,1,1,0,0,1, 0,1,1,1,1,1,1,0,1,0,0,0,0,0,0,1, 0,0,0,1,1,0,0,0,1,1,1,0,0,1,1,1, 0,0,0,0,1,1,1,1,0,0,1,1,0,0,1,1, 0,0,1,1,0,0,1,1,1,1,1,1,0,0,0,0, 0,0,1,0,0,0,1,0,1,1,1,0,1,1,1,0, 0,1,0,0,0,1,0,0,0,1,1,1,0,1,1,1,
];

#[rustfmt::skip]
const TABLE_ANCHOR_INDEX_SECOND_SUBSET: [u8; 64] = [
	15,15,15,15,15,15,15,15,
	15,15,15,15,15,15,15,15,
	15, 2, 8, 2, 2, 8, 8,15,
	 2, 8, 2, 2, 8, 8, 2, 2,
	15,15, 6, 8, 2, 8,15,15,
	 2, 8, 2, 2, 2,15,15, 6,
	 6, 2, 6, 8,15,15, 2, 2,
	15,15,15,15,15, 2, 2,15,
];

const WEIGHTS2: [u8; 4] = [0, 21, 43, 64];
const WEIGHTS3: [u8; 8] = [0, 9, 18, 27, 37, 46, 55, 64];
const WEIGHTS4: [u8; 16] = [0, 4, 9, 13, 17, 21, 26, 30, 34, 38, 43, 47, 51, 55, 60, 64];

pub fn decode(data: &[u8]) -> Result<ColorBlock, Bc7Error> {
	debug_assert_eq!(data.len(), 16);

	let mode = data[0].trailing_zeros() as u8;
	// trace!(mode, "Decoding BC7");

	match mode {
		0 | 2 => Err(Bc7Error::UnimplementedBc7BlockMode(mode)),
		1 | 3 | 7 => Ok(decode_mode_1_3_7(data, mode)),
		4 | 5 => Ok(decode_mode_4_5(data, mode)),
		6 => Ok(decode_mode_6(data)),
		_ => Err(Bc7Error::InvalidBc7BlockMode(mode)),
	}
}

fn decode_mode_1_3_7(data: &[u8], mode: u8) -> ColorBlock {
	debug_assert!(matches!(mode, 1 | 3 | 7));

	const ENDPOINTS: usize = 4;
	let comps: usize = if mode == 7 { 4 } else { 3 };
	let weight_bits: usize = if mode == 1 { 3 } else { 2 };
	let endpoint_bits: usize = match mode {
		1 => 6,
		3 => 7,
		7 => 5,
		_ => unreachable!(),
	};
	let pbits_count: usize = if mode == 1 { 2 } else { 4 };
	let shared_pbits: bool = mode == 1;
	let weight_vals: usize = 1 << weight_bits;

	let mut bit_offset = 0;

	let read_mode = read_bits32(data, &mut bit_offset, mode as usize + 1).trailing_zeros() as u8;
	debug_assert_eq!(read_mode, mode);

	let part = read_bits32(data, &mut bit_offset, 6) as usize;

	let mut endpoints = [Color4::default(); ENDPOINTS];
	for c in 0..comps {
		for e in 0..ENDPOINTS {
			endpoints[e].components[c] = read_bits32(data, &mut bit_offset, endpoint_bits) as u8;
		}
	}
	// tracing::trace!(?endpoints);

	let mut pbits = [0usize; 4];
	for p in 0..pbits_count {
		pbits[p] = read_bits32(data, &mut bit_offset, 1) as usize;
	}
	// tracing::trace!(?pbits);

	let mut weights = [0usize; 16];
	for i in 0..16 {
		let bits = if (i == 0) || (i as u8 == TABLE_ANCHOR_INDEX_SECOND_SUBSET[part]) {
			weight_bits - 1
		} else {
			weight_bits
		};
		weights[i] = read_bits32(data, &mut bit_offset, bits) as usize;
	}
	// tracing::trace!(?weights);

	debug_assert_eq!(bit_offset, 128);

	for e in 0..ENDPOINTS {
		for c in 0..4 {
			endpoints[e].components[c] = if c == comps {
				255
			} else {
				bc7_dequant_with_pbit(
					endpoints[e].components[c],
					pbits[if shared_pbits { e >> 1 } else { e }],
					endpoint_bits,
				)
			};
		}
	}

	let mut block_colors = [[Color4::default(); 8]; 2];
	for s in 0..2 {
		for i in 0..weight_vals {
			for c in 0..comps {
				block_colors[s][i].components[c] = bc7_interp(
					endpoints[s * 2].components[c],
					endpoints[s * 2 + 1].components[c],
					i,
					weight_bits,
				);
			}
			block_colors[s][i].components[3] = if comps == 3 {
				255
			} else {
				block_colors[s][i].components[3]
			};
		}
	}

	let mut ret = [[Color4::default(); 4]; 4];

	for i in 0..16 {
		let x = i & 3;
		let y = i >> 2;

		ret[y][x] = block_colors[PARTITION2[part * 16 + i] as usize][weights[i]];
	}

	ret
}

fn decode_mode_4_5(data: &[u8], mode: u8) -> ColorBlock {
	debug_assert!(mode == 4 || mode == 5);

	const ENDPOINTS: usize = 2;
	const COMPS: usize = 4;
	let color_endpoint_bits: usize = if mode == 4 { 5 } else { 7 }; // ColorComponentPrecision
	let alpha_endpoint_bits: usize = if mode == 4 { 6 } else { 8 }; // AlphaComponentPrecision

	let mut bit_offset = 0;

	let read_mode = read_bits32(data, &mut bit_offset, mode as usize + 1).trailing_zeros() as u8;
	debug_assert_eq!(read_mode, mode);

	let comp_rot = read_bits32(data, &mut bit_offset, 2) as usize;

	// if index_mode is 1, color will use 3 bits and alpha will use 2 bits instead of the other way around
	// mode 5 always uses 2 bits for both color and alpha
	let index_mode = if mode == 4 {
		read_bits32(data, &mut bit_offset, 1) as usize
	} else {
		0
	};

	// trace!(?read_mode, ?comp_rot, ?index_mode);

	let mut endpoints = [Color4::default(); 2];
	for c in 0..COMPS {
		for e in 0..ENDPOINTS {
			let bits = if c == 3 {
				alpha_endpoint_bits
			} else {
				color_endpoint_bits
			};
			let color = read_bits32(data, &mut bit_offset, bits) as u8;
			endpoints[e].components[c] = color;
		}
	}
	// tracing::trace!(?endpoints);

	// ColorIndexBitCount and AlphaIndexBitCount
	let weight_bits = [
		if index_mode == 1 && mode == 4 { 3 } else { 2 },
		if index_mode == 0 && mode == 4 { 3 } else { 2 },
	];

	// weights and a_weights
	let mut weights = [[0u8; 16]; 2];

	for i in 0..16 {
		weights[index_mode][i] = read_bits32(
			data,
			&mut bit_offset,
			weight_bits[index_mode] - if i == 0 { 1 } else { 0 },
		) as u8;
	}
	for i in 0..16 {
		weights[1 - index_mode][i] = read_bits32(
			data,
			&mut bit_offset,
			weight_bits[1 - index_mode] - if i == 0 { 1 } else { 0 },
		) as u8;
	}
	// trace!(?weights);

	debug_assert_eq!(bit_offset, 128);

	for e in 0..ENDPOINTS {
		for c in 0..COMPS {
			endpoints[e].components[c] = bc7_dequant(
				endpoints[e].components[c],
				if c == 3 {
					alpha_endpoint_bits
				} else {
					color_endpoint_bits
				},
			);
		}
	}
	// tracing::trace!(?endpoints);

	// block colors
	let mut block_colors = [Color4::default(); 8];
	for i in 0..1 << weight_bits[0] {
		for c in 0..3 {
			block_colors[i].components[c] = bc7_interp(
				endpoints[0].components[c],
				endpoints[1].components[c],
				i,
				weight_bits[0],
			);
		}
	}
	for i in 0..1 << weight_bits[1] {
		block_colors[i].components[3] = bc7_interp(
			endpoints[0].components[3],
			endpoints[1].components[3],
			i,
			weight_bits[1],
		);
	}
	// trace!(?block_colors);

	let mut ret = [[Color4::default(); 4]; 4];
	for i in 0..16 {
		let x = i & 0b11;
		let y = i >> 2;

		ret[y][x] = block_colors[weights[0][i] as usize];
		ret[y][x].components[3] = block_colors[weights[1][i] as usize].components[3];

		if comp_rot >= 1 {
			ret[y][x].components.swap(3, comp_rot - 1);
		}
	}

	ret
}

fn decode_mode_6(data: &[u8]) -> ColorBlock {
	let data_lo = u64::from_le_bytes(data[0..8].try_into().expect("data is not 16 bytes"));
	let data_hi = u64::from_le_bytes(data[8..16].try_into().expect("data is not 16 bytes"));

	let get_bits_lo = |bit_idx: usize, bit_len: usize| data_lo >> bit_idx & ((1 << bit_len) - 1);
	let get_bits_hi = |bit_idx: usize, bit_len: usize| data_hi >> bit_idx & ((1 << bit_len) - 1);

	let mode = get_bits_lo(0, 7);
	debug_assert_eq!(mode, 1 << 6);

	let r0 = (get_bits_lo(7, 7) << 1) | get_bits_lo(63, 1);
	let g0 = (get_bits_lo(7 * 3, 7) << 1) | get_bits_lo(63, 1);
	let b0 = (get_bits_lo(7 * 5, 7) << 1) | get_bits_lo(63, 1);
	let a0 = (get_bits_lo(7 * 7, 7) << 1) | get_bits_lo(63, 1);

	let r1 = (get_bits_lo(7 * 2, 7) << 1) | get_bits_hi(0, 1);
	let g1 = (get_bits_lo(7 * 4, 7) << 1) | get_bits_hi(0, 1);
	let b1 = (get_bits_lo(7 * 6, 7) << 1) | get_bits_hi(0, 1);
	let a1 = (get_bits_lo(7 * 8, 7) << 1) | get_bits_hi(0, 1);

	let mut vals = [Color4::default(); 16];
	for i in 0..16 {
		let w = WEIGHTS4[i] as u64;
		let iw = 64 - w;
		vals[i] = Color4 {
			components: [
				((r0 * iw + r1 * w + 32) >> 6) as u8,
				((g0 * iw + g1 * w + 32) >> 6) as u8,
				((b0 * iw + b1 * w + 32) >> 6) as u8,
				((a0 * iw + a1 * w + 32) >> 6) as u8,
			],
		};
	}
	// trace!(?vals);

	let mut ret = [[Color4::default(); 4]; 4];
	ret[0][0] = vals[get_bits_hi(1, 3) as usize];
	ret[0][1] = vals[get_bits_hi(4, 4) as usize];
	ret[0][2] = vals[get_bits_hi(4 * 2, 4) as usize];
	ret[0][3] = vals[get_bits_hi(4 * 3, 4) as usize];
	ret[1][0] = vals[get_bits_hi(4 * 4, 4) as usize];
	ret[1][1] = vals[get_bits_hi(4 * 5, 4) as usize];
	ret[1][2] = vals[get_bits_hi(4 * 6, 4) as usize];
	ret[1][3] = vals[get_bits_hi(4 * 7, 4) as usize];
	ret[2][0] = vals[get_bits_hi(4 * 8, 4) as usize];
	ret[2][1] = vals[get_bits_hi(4 * 9, 4) as usize];
	ret[2][2] = vals[get_bits_hi(4 * 10, 4) as usize];
	ret[2][3] = vals[get_bits_hi(4 * 11, 4) as usize];
	ret[3][0] = vals[get_bits_hi(4 * 12, 4) as usize];
	ret[3][1] = vals[get_bits_hi(4 * 13, 4) as usize];
	ret[3][2] = vals[get_bits_hi(4 * 14, 4) as usize];
	ret[3][3] = vals[get_bits_hi(4 * 15, 4) as usize];

	ret
}

fn read_bits32(data: &[u8], bit_offset: &mut usize, codesize: usize) -> u32 {
	debug_assert!(codesize <= 32);
	let mut bits = 0;
	let mut total_bits = 0;

	while total_bits < codesize {
		let byte_bit_offset = *bit_offset & 7;
		let bits_to_read = std::cmp::min(codesize - total_bits, 8 - byte_bit_offset);

		let byte_bits = (data[*bit_offset >> 3] >> byte_bit_offset) as u32;
		let mask = (1 << bits_to_read) - 1;
		let byte_bits = byte_bits & mask;

		bits |= byte_bits << total_bits;

		total_bits += bits_to_read;
		*bit_offset += bits_to_read;
	}

	bits
}

fn bc7_dequant(val: u8, val_bits: usize) -> u8 {
	let val = val as usize;

	debug_assert!(val < (1 << val_bits));
	debug_assert!((4..=8).contains(&val_bits));

	let mut val = val << (8 - val_bits);
	val |= val >> val_bits;
	debug_assert!(val <= 255);

	val as u8
}

fn bc7_dequant_with_pbit(val: u8, pbit: usize, val_bits: usize) -> u8 {
	let val = val as usize;

	debug_assert!(val < (1 << val_bits));
	debug_assert!((4..=8).contains(&val_bits));
	debug_assert!(pbit < 2);

	let total_bits = val_bits + 1;
	let mut val = (val << 1) | pbit;
	val <<= 8 - total_bits;
	val |= val >> total_bits;
	debug_assert!(val <= 255);

	val as u8
}

fn bc7_interp2(l: u8, h: u8, w: usize) -> u8 {
	debug_assert!(w < 4);
	let l = l as usize;
	let h = h as usize;
	((l * (64 - WEIGHTS2[w]) as usize + h * WEIGHTS2[w] as usize + 32) >> 6) as u8
}

fn bc7_interp3(l: u8, h: u8, w: usize) -> u8 {
	debug_assert!(w < 8);
	let l = l as usize;
	let h = h as usize;
	((l * (64 - WEIGHTS3[w]) as usize + h * WEIGHTS3[w] as usize + 32) >> 6) as u8
}

fn bc7_interp4(l: u8, h: u8, w: usize) -> u8 {
	debug_assert!(w < 16);
	let l = l as usize;
	let h = h as usize;
	((l * (64 - WEIGHTS4[w]) as usize + h * WEIGHTS4[w] as usize + 32) >> 6) as u8
}

fn bc7_interp(l: u8, h: u8, w: usize, bits: usize) -> u8 {
	match bits {
		2 => bc7_interp2(l, h, w),
		3 => bc7_interp3(l, h, w),
		4 => bc7_interp4(l, h, w),
		_ => unreachable!("bad bits value in bc_interp"),
	}
}

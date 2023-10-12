use crate::Color4;

/// Unpacks a u16 color value into an RGBA color. Note that the alpha component is always 255 and it
/// is read as BGR, not RGB.
pub fn unpack_dxt_color_565(color: u16) -> Color4 {
	Color4 {
		components: [
			(((color >> 11) & 0b1_1111) << 3) as u8,
			(((color >> 5) & 0b11_1111) << 2) as u8,
			((color & 0b1_1111) << 3) as u8,
			255,
		],
	}
}

/// Calculate `color_2` as calculated by `(2 * color_0 + color_1 + 1) / 3`.
pub fn interp_color_2_opaque(color_0: Color4, color_1: Color4) -> Color4 {
	Color4 {
		components: [
			((2 * color_0.components[0] as usize + color_1.components[0] as usize + 1) / 3) as u8,
			((2 * color_0.components[1] as usize + color_1.components[1] as usize + 1) / 3) as u8,
			((2 * color_0.components[2] as usize + color_1.components[2] as usize + 1) / 3) as u8,
			((2 * color_0.components[3] as usize + color_1.components[3] as usize + 1) / 3) as u8,
		],
	}
}

/// Calculate `color_3` as calculated by `(color_0 + 2 * color_1 + 1) / 3`.
pub fn interp_color_3_opaque(color_0: Color4, color_1: Color4) -> Color4 {
	Color4 {
		components: [
			((color_0.components[0] as usize + 2 * color_1.components[0] as usize + 1) / 3) as u8,
			((color_0.components[1] as usize + 2 * color_1.components[1] as usize + 1) / 3) as u8,
			((color_0.components[2] as usize + 2 * color_1.components[2] as usize + 1) / 3) as u8,
			((color_0.components[3] as usize + 2 * color_1.components[3] as usize + 1) / 3) as u8,
		],
	}
}

/// Calculate `color_2` as calculated by `(color_0 + color_1) / 2`.
pub fn interp_color_2_transparent(color_0: Color4, color_1: Color4) -> Color4 {
	Color4 {
		components: [
			((color_0.components[0] as usize + color_1.components[0] as usize) / 2) as u8,
			((color_0.components[1] as usize + color_1.components[1] as usize) / 2) as u8,
			((color_0.components[2] as usize + color_1.components[2] as usize) / 2) as u8,
			((color_0.components[3] as usize + color_1.components[3] as usize) / 2) as u8,
		],
	}
}

/// "Calculate" `color_3`, which is just transparent.
pub fn interp_color_3_transparent(_: Color4, _: Color4) -> Color4 {
	Color4::TRANSPARENT
}

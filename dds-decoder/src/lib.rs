use errors::DdsDecodeError;

mod bc7;
pub mod errors;

pub fn decode_image(
	format: DdsFormat,
	data: &[u8],
	width: usize,
	height: usize,
) -> Result<Vec<u8>, DdsDecodeError> {
	match format {
		DdsFormat::BC7 => Ok(bc7::read_image(data, width, height)?),
		_ => Err(DdsDecodeError::UnsupportedFormat(format)),
	}
}

#[derive(Default, Debug, Copy, Clone)]
pub struct Color4(pub [u8; 4]);

pub struct DecodedImage {
	width: usize,
	pub data: Vec<Color4>,
}

impl DecodedImage {
	pub fn width(&self) -> usize {
		self.width
	}

	pub fn height(&self) -> usize {
		debug_assert_eq!(self.data.len() % self.width, 0);
		self.data.len() / self.width
	}
}

#[derive(Debug, Copy, Clone)]
pub enum DdsFormat {
	/// Uncompressed RGBA8
	RGBA8,
	/// BC1/DXT1: Three-channel color with alpha channel.
	///
	/// Uses DDS magic "DXT1"
	BC1,
	/// BC3/DXT5: Three-channel color with alpha channel.
	///
	/// Uses DDS magic "DXT5"
	BC3,
	/// BC6H: Three-channel high dynamic range (HDR) color.
	BC6H,
	/// BC7: Three-channel color, alpha channel optional.
	BC7,
}

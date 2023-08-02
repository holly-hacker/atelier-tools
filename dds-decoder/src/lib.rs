mod bc7;
mod errors;

pub fn decode_image(format: DdsFormat, data: &[u8], width: usize, height: usize) -> Vec<u8> {
	match format {
		DdsFormat::BC7 => bc7::read_image(data, width, height),
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
	BC7,
}

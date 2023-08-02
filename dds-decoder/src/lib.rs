mod bc7;

pub use bc7::read_image as read_bc7_image;

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

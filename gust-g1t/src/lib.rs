mod bc7;
pub mod errors;
mod util;

use std::io::{Read, Seek};

use errors::G1tReadError;
use scroll::IOread;
use tracing::{debug, trace, warn};
use ux::u4;

use crate::bc7::Color4;

pub struct GustG1t {
	#[allow(unused)]
	header: G1tHeader,

	// TODO: this is awful, but I don't want to deal with the complexities of g1t right now
	texture_type: u8,
	pub height: u32,
	pub width: u32,
	offset: u64,
}

impl GustG1t {
	pub fn read(mut reader: impl Read + Seek) -> Result<Self, G1tReadError> {
		let (header, global_flags) = G1tHeader::read(&mut reader)?;
		debug_assert_eq!(header.texture_count, global_flags.len() as u32);

		trace!(
			"position after reading header: {}",
			reader.stream_position()?
		);
		trace!(?header);
		debug!("Texture count: {}", header.texture_count);
		debug!("Platform: {:?}", header.platform);

		// read offset table
		trace!("Seeking to {} for offset table", header.header_size);
		reader.seek(std::io::SeekFrom::Start(header.header_size as u64))?;
		let mut offsets = Vec::with_capacity(header.texture_count as usize);
		for _ in 0..header.texture_count {
			let offset: u32 = reader.ioread()?;
			offsets.push(offset);
		}
		trace!(?offsets);

		if offsets.len() > 1 {
			todo!("support multiple textures");
		}

		// read textures
		// this should be a loop instead of an if, but we only support 1 texture for now
		if let Some((index, offset)) = offsets.into_iter().enumerate().next() {
			let span = tracing::debug_span!("texture", index, offset);
			let _guard = span.enter();

			let _global_flag = global_flags[index];

			reader.seek(std::io::SeekFrom::Start(
				(header.header_size + offset) as u64,
			))?;
			let texture_header = G1tTextureHeader::read(&mut reader)?;
			trace!(?texture_header);

			if texture_header.texture_type != 0x5F {
				todo!("only texture type 0x5F is supported (BC7)");
			}

			// header has been read, image data comes now
			let mut width = texture_header.width();
			let mut height = texture_header.height();
			let mut frames = 1;

			// possible extended data section
			if texture_header.flags.contains(TextureFlags::EXTENDED_DATA) {
				let extended_data_len: u32 = reader.ioread()?;
				trace!(?extended_data_len);

				const HEADER_SIZE_FRAMES_DEPTH: u32 = 0x0C;
				const HEADER_SIZE_NON_STANDARD_WIDTH: u32 = 0x10;
				const HEADER_SIZE_NON_STANDARD_HEIGHT: u32 = 0x14;

				if !matches!(
					extended_data_len,
					HEADER_SIZE_FRAMES_DEPTH
						| HEADER_SIZE_NON_STANDARD_WIDTH
						| HEADER_SIZE_NON_STANDARD_HEIGHT
				) {
					return Err(G1tReadError::InvalidExtendedDataSize(extended_data_len));
				}

				if extended_data_len >= HEADER_SIZE_FRAMES_DEPTH {
					let depth: u32 = reader.ioread()?;
					let texture_flags_2: u32 = reader.ioread()?;
					// TODO: frames?
					let frames_from_flags =
						((texture_flags_2 >> 28) & 0x0F) + ((texture_flags_2 >> 12) & 0xF0);
					frames = if frames_from_flags == 0 {
						1
					} else {
						frames_from_flags
					};

					trace!(?depth, ?texture_flags_2, ?frames_from_flags);
				}

				if extended_data_len >= HEADER_SIZE_NON_STANDARD_WIDTH {
					width = reader.ioread()?;
					trace!(?width, "non-standard width");
				}

				if extended_data_len >= HEADER_SIZE_NON_STANDARD_HEIGHT {
					height = reader.ioread()?;
					trace!(?height, "non-standard height");
				}
			}

			if frames > 1 {
				panic!("Texture has {} frames, only 1 is supported for now", frames);
			}

			// need to create some kind of structure data here?
			return Ok(Self {
				header,
				texture_type: texture_header.texture_type,
				height,
				width,
				offset: reader.stream_position()?,
			});
		}

		todo!("handle no textures");
	}

	pub fn read_image(&self, mut reader: impl Read + Seek) -> Result<Vec<u8>, G1tReadError> {
		// size for BC7
		// assuming mipmap level 0
		let blocks_x = usize::max(1, (self.width as usize + 3) / 4);
		let blocks_y = usize::max(1, (self.height as usize + 3) / 4);
		let block_count = blocks_x * blocks_y;
		let encoded_size = block_count * 16;
		let pixel_count = encoded_size; // BC7 has a compression ratio of exactly 1:4, so this lines up for RGBA8
		debug!(?encoded_size, "Size of encoded image data");

		reader.seek(std::io::SeekFrom::Start(self.offset))?;

		let mut data = vec![0u8; encoded_size];
		reader.read_exact(&mut data)?;

		debug!(len = data.len(), "Data read");

		// assume 128 bits are read at once (16 bytes)
		// the size of each chunk is a 4x4 block of rgba8 pixels
		let mut decoded_pixels = vec![Color4::default(); pixel_count];
		for (chunk_index, chunk) in data.chunks(16).enumerate() {
			// let pos = self.offset + (index as u64 * 16);
			// let span = tracing::debug_span!("chunk", index, pos = format!("0x{pos:x}"));
			// let _guard = span.enter();
			let chunk = bc7::decode(chunk);

			// copy the chunk into the decoded pixels
			let chunk_x = (chunk_index % blocks_x) * 4;
			let chunk_y = (chunk_index / blocks_x) * 4;
			let target_index = chunk_y * (blocks_x * 4) + chunk_x;
			for y in 0..4 {
				for x in 0..4 {
					let target_index = target_index + y * (blocks_x * 4) + x;
					decoded_pixels[target_index] = chunk[y][x];
				}
			}
		}

		// TODO: this could still be too big if the original image dimensions are not divisible by 4!
		let decoded_pixels = decoded_pixels
			.into_iter()
			.flat_map(|color| color.0)
			.collect::<Vec<_>>();

		debug!("image decoded");

		// todo!("decoding done")
		Ok(decoded_pixels)
	}
}

#[derive(Debug)]
pub struct G1tHeader {
	pub version: u16,
	pub header_size: u32,
	pub texture_count: u32,
	pub platform: Platform,
	pub extra_size: u32,
}

impl G1tHeader {
	const MAGIC_LITTLE_ENDIAN: u32 = u32::from_be_bytes(*b"G1TG");
	const MAGIC_BIG_ENDIAN: u32 = u32::from_le_bytes(*b"G1TG");

	fn read(mut reader: impl Read + Seek) -> Result<(Self, Vec<GlobalTextureFlags>), G1tReadError> {
		let magic = reader.ioread()?;
		let version_string: u32 = reader.ioread()?;
		let total_size: u32 = reader.ioread()?;
		let header_size = reader.ioread()?;
		let texture_count = reader.ioread()?;
		let platform: u32 = reader.ioread()?;
		let extra_size = reader.ioread()?;

		match magic {
			Self::MAGIC_LITTLE_ENDIAN => {}
			Self::MAGIC_BIG_ENDIAN => todo!("support big endian g1t files"),
			_ => return Err(G1tReadError::InvalidHeaderMagic(magic)),
		}

		// the version is ascii, presumably decimal
		// some known values:
		// - 30303630 (0060): Atelier Ryza 1
		// - 30303634 (0064): Atelier Ryza 2
		// - 30303634 (0064): Atelier Ryza 3
		let version: u16 = String::from_utf8(version_string.to_be_bytes().to_vec())?
			.parse()
			.map_err(G1tReadError::VersionParseError)?;

		if !matches!(version / 100, 0 | 1) {
			warn!("Potentially unsupported g1t version: {}", version);
		}

		// TODO: perhaps allow opening faulty files with a flag?
		let real_file_size = util::stream_len(&mut reader)?;
		if total_size != real_file_size as u32 {
			return Err(G1tReadError::InvalidTotalSize(
				total_size,
				real_file_size as u32,
			));
		}

		let platform = Platform::from_repr(platform as usize)
			.ok_or_else(|| G1tReadError::UnknownPlatform(platform))?;

		if extra_size > 0xFFFF || extra_size % 4 != 0 {
			return Err(G1tReadError::InvalidExtraSize(extra_size));
		}

		// TODO: global flags should be saved
		let global_flags = (0..texture_count)
			.map(|_| {
				reader
					.ioread::<u32>()
					.map(GlobalTextureFlags::from_bits_retain)
			})
			.collect::<Result<Vec<_>, _>>()?;
		debug!(?global_flags);

		Ok((
			Self {
				version,
				header_size,
				texture_count,
				platform,
				extra_size,
			},
			global_flags,
		))
	}
}

#[derive(custom_debug::Debug)]
struct G1tTextureHeader {
	#[debug(format = "{}")]
	z_mipmaps: u4,
	#[debug(format = "{}")]
	mipmaps: u4,
	texture_type: u8,
	/// X size, as a power of 2
	#[debug(format = "{}")]
	dx: u4,
	/// Y size, as a power of 2
	#[debug(format = "{}")]
	dy: u4,
	flags: TextureFlags,
}

impl G1tTextureHeader {
	fn read(mut reader: impl Read + Seek) -> Result<Self, G1tReadError> {
		let packed_mipmaps: u8 = reader.ioread()?;
		let r#type = reader.ioread()?;
		let packed_dimensions: u8 = reader.ioread()?;
		let mut flags = [0u8; 8];
		reader.read_exact(&mut flags[3..8])?;
		let flags = TextureFlags::from_bits_retain(u64::from_be_bytes(flags));

		let (z_mipmaps, mipmaps) = (
			u4::new(packed_mipmaps & 0x0F),
			u4::new((packed_mipmaps & 0xF0) >> 4),
		);
		let (dx, dy) = (
			u4::new(packed_dimensions & 0x0F),
			u4::new((packed_dimensions & 0xF0) >> 4),
		);

		if mipmaps == u4::new(0) {
			return Err(G1tReadError::NoMipmaps);
		}

		Ok(Self {
			z_mipmaps,
			mipmaps,
			texture_type: r#type,
			dx,
			dy,
			flags,
		})
	}

	pub fn width(&self) -> u32 {
		1 << (<u4 as Into<u32>>::into(self.dx))
	}

	pub fn height(&self) -> u32 {
		// TODO: also depends on DOUBLE_HEIGHT flag
		1 << (<u4 as Into<u32>>::into(self.dy))
	}
}

bitflags::bitflags! {
	#[derive(Debug, Copy, Clone)]
	struct GlobalTextureFlags: u32 {
		const NORMAL_MAP = 0x00_00_03;
	}
}

bitflags::bitflags! {
	#[derive(Debug, Copy, Clone)]
	struct TextureFlags: u64 {
		// NOTE: gust_tools shifts all nibbles of the texture flags around! we don't do this

		const STANDARD_FLAGS = 0x00_00_10_21_00;

		const EXTENDED_DATA = 0x00_00_00_00_10;
		const DOUBLE_HEIGHT = 0x01_00_00_00_00;
	}
}

#[derive(Debug, Copy, Clone, strum::FromRepr)]
pub enum Platform {
	/// Sony PlayStation 2
	PlayStation2 = 0x00,
	/// Sony PlayStation 3
	PlayStation3 = 0x01,
	/// Microsoft XBOX 360
	Xbox360 = 0x02,
	/// Nintendo Wii
	Wii = 0x03,
	/// Nintendo DS
	NintendoDs = 0x04,
	/// Nintendo 3DS
	Nintendo3ds = 0x05,
	/// Sony PlayStation Vita
	PlayStationVita = 0x06,
	/// Android
	Android = 0x07,
	/// Apple iOS
	Ios = 0x08,
	/// Nintendo Wii U
	WiiU = 0x09,
	/// Windows
	Windows = 0x0A,
	/// Sony PlayStation 4
	PlayStation4 = 0x0B,
	/// Microsoft XBOX One
	XboxOne = 0x0C,
	// missing: 0x0D, 0x0E, 0x0F. perhaps unused?
	/// Nintendo Switch
	Switch = 0x10,
}

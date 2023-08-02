pub mod errors;
mod util;

use std::io::{Read, Seek};

use errors::G1tReadError;
use scroll::IOread;
use tracing::{debug, trace, warn};

pub struct GustG1t {
	#[allow(unused)]
	header: G1tHeader,

	pub textures: Vec<TextureInfo>,
}

/// A single texture in a g1t file.
pub struct TextureInfo {
	header: G1tTextureHeader,
	#[allow(unused)]
	global_flag: GlobalTextureFlags,
	pub height: u32,
	pub width: u32,
	frames: u32,
	absolute_data_offset: u64,
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

		// read textures
		let mut textures = vec![];
		if let Some((texture_index, offset)) = offsets.into_iter().enumerate().next() {
			let span = tracing::debug_span!("texture", texture_index, offset);
			let _guard = span.enter();

			let global_flag = global_flags[texture_index];

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

			// need to create some kind of structure data here?
			let texture_info = TextureInfo {
				header: texture_header,
				global_flag,
				height,
				width,
				frames,
				absolute_data_offset: reader.stream_position()?,
			};
			textures.push(texture_info);
		}

		Ok(Self { header, textures })
	}

	pub fn read_image(
		&self,
		texture: &TextureInfo,
		mut reader: impl Read + Seek,
	) -> Result<Vec<u8>, G1tReadError> {
		if texture.header.texture_type != 0x5F {
			todo!("Only BC7 textures are supported for now");
		}

		if texture.header.mipmaps > 1 {
			todo!("Mipmaps are not supported for now");
		}

		if texture.header.z_mipmaps > 1 {
			todo!("Z-mipmaps are not supported for now");
		}

		if texture.frames > 1 {
			todo!(
				"Texture has {} frames, only 1 is supported for now",
				texture.frames
			);
		}

		match texture.header.texture_type {
			0x5F => {
				// assuming mipmap level 0
				let blocks_x = usize::max(1, (texture.width as usize + 3) / 4);
				let blocks_y = usize::max(1, (texture.height as usize + 3) / 4);
				let encoded_data_size = blocks_x * blocks_y * 16;
				debug!(?encoded_data_size, "Size of encoded image data");

				reader.seek(std::io::SeekFrom::Start(texture.absolute_data_offset))?;

				let mut data = vec![0u8; encoded_data_size];
				reader.read_exact(&mut data)?;
				debug!(len = data.len(), "Data read");

				Ok(dds_decoder::read_bc7_image(
					&data,
					texture.width as usize,
					texture.height as usize,
				))
			}
			_ => todo!("Only BC7 textures are supported for now"),
		}
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

#[derive(Debug)]
struct G1tTextureHeader {
	z_mipmaps: u8,
	mipmaps: u8,
	texture_type: u8,
	/// X size, as a power of 2
	dx: u8,
	/// Y size, as a power of 2
	dy: u8,
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

		let (z_mipmaps, mipmaps) = (packed_mipmaps & 0x0F, ((packed_mipmaps & 0xF0) >> 4));
		let (dx, dy) = (
			(packed_dimensions & 0x0F),
			((packed_dimensions & 0xF0) >> 4),
		);

		if mipmaps == (0) {
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
		1 << self.dx
	}

	pub fn height(&self) -> u32 {
		// TODO: also depends on DOUBLE_HEIGHT flag
		1 << self.dy
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

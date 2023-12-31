use errors::PakReadError;
use gust_common::GameVersion;
use scroll::IOread;
use std::{
	ffi::CStr,
	io::{self, Read, Seek},
};
use tracing::{debug, trace, warn};
use utils::XorReader;

pub use gust_common as common;

use crate::utils::FencedReader;

mod errors;
mod utils;

/// A representation of the contents of a .pak file. This does not include the file data itself, but
/// can be used to read the file data.
pub struct GustPak {
	#[allow(unused)]
	header: PakHeader,

	/// The file entries in the .pak file.
	pub entries: PakEntryList,

	/// the offset at which the entries begin.
	data_start: u64,
}

impl GustPak {
	/// Reads a .pak file from a reader.
	pub fn read_index(
		mut reader: impl Read + Seek,
		game_version: GameVersion,
	) -> Result<Self, PakReadError> {
		let pak_type = Self::get_pak_type(game_version);
		let pak_key = Self::get_pak_key(game_version);
		debug!("Using game version: {:?}", game_version);
		debug!("Using pak type: {:?}", pak_type);

		let header = PakHeader::read(&mut reader)?;

		trace!(?header);

		let entries = match pak_type {
			PakEntryType::Entry32 => {
				debug_assert_eq!(pak_key, None);
				let mut entries: Vec<Entry32> = Vec::with_capacity(header.file_count as usize);

				for _ in 0..header.file_count {
					let entry = Entry32::read(&mut reader)?;
					trace!(?entry);
					entries.push(entry);
				}

				PakEntryList::Entry32(entries)
			}
			PakEntryType::Entry64 => {
				debug_assert_eq!(pak_key, None);
				let mut entries: Vec<Entry64> = Vec::with_capacity(header.file_count as usize);

				for _ in 0..header.file_count {
					let entry = Entry64::read(&mut reader)?;
					trace!(?entry);
					entries.push(entry);
				}

				PakEntryList::Entry64(entries)
			}
			PakEntryType::Entry64Ext => {
				let mut entries: Vec<Entry64Ext> = Vec::with_capacity(header.file_count as usize);

				for _ in 0..header.file_count {
					let entry = Entry64Ext::read(&mut reader, pak_key)?;
					trace!(?entry);
					entries.push(entry);
				}

				PakEntryList::Entry64Ext(entries)
			}
		};

		let data_start = reader.stream_position()?;

		if entries.len() != header.file_count as usize {
			warn!(
				"Header claims {} entries but read {}",
				header.file_count,
				entries.len()
			);
		}

		Ok(Self {
			header,
			entries,
			data_start,
		})
	}

	pub fn get_data_start(&self) -> u64 {
		self.data_start
	}

	fn get_pak_type(version: GameVersion) -> PakEntryType {
		match version {
			GameVersion::A17 => PakEntryType::Entry32,
			// Starting from A18
			GameVersion::A18 | GameVersion::A19 | GameVersion::A21 => PakEntryType::Entry64,
			// Starting from A22
			GameVersion::A22 | GameVersion::A23 | GameVersion::A24 => PakEntryType::Entry64Ext,
		}
	}

	/// Returns the key used to decrypt the .pak file.
	fn get_pak_key(version: GameVersion) -> Option<&'static [u8; 32]> {
		// Only games starting from A23 use a key.
		// These keys look like base64 but are interpreted as an ascii xor key.
		match version {
			GameVersion::A17
			| GameVersion::A18
			| GameVersion::A19
			| GameVersion::A21
			| GameVersion::A22 => None,
			GameVersion::A23 => Some(b"dGGKXLHLuCJwv8aBc3YQX6X6sREVPchs"),
			GameVersion::A24 => Some(b"fyrixtT9AhA4v0cFahgMcgVwxFrry42A"),
		}
	}

	/// Decrypts some data in-place.
	fn decrypt(ciphertext: &mut [u8], file_key: &[u8], pak_key: Option<&[u8; 32]>) {
		// default to null bytes if no pak_key was given
		let mut pak_key = pak_key.cloned().unwrap_or_default();

		debug_assert!(
			file_key.len() <= pak_key.len(),
			"file key may not be larger than pak key"
		);

		// xor pak_key with file_key to get xor key
		pak_key.iter_mut().enumerate().for_each(|(i, b)| {
			*b ^= file_key[i % file_key.len()];
		});

		let xor_key = &pak_key[..file_key.len()];
		// trace!("Decrypting name with xor key: {:?}", xor_key);

		// xor ciphertext with xor_key
		ciphertext.iter_mut().enumerate().for_each(|(i, b)| {
			*b ^= xor_key[i % xor_key.len()];
		});
	}
}

#[derive(custom_debug::Debug)]
struct PakHeader {
	file_count: u32,
	flags: u32,
}

impl PakHeader {
	fn read(mut reader: impl Read) -> Result<Self, PakReadError> {
		let version = reader.ioread()?;
		let file_count = reader.ioread()?;
		let header_size = reader.ioread()?;
		let flags = reader.ioread()?;

		if version != 0x20000 {
			return Err(PakReadError::InvalidHeaderVersion(version));
		}

		if header_size != 16 {
			return Err(PakReadError::InvalidHeaderVersion(header_size));
		}

		if file_count > 0x10000 {
			return Err(PakReadError::TooManyFiles(file_count));
		}

		Ok(Self { file_count, flags })
	}
}

// Not present: Entry32 used by A17 (Atelier Sophie)

/// File entries for games up to and including A17 (Atelier Sophie)
#[derive(custom_debug::Debug, Clone)]
pub struct Entry32 {
	file_name: String,
	#[debug(format = "{:#x}")]
	file_size: u32,
	/// The xor key used to decrypt the file name and content.
	file_key: [u8; 20],
	/// The data offset of this file's data in the .pak file. This is not the file offset.
	#[debug(format = "{:#x}")]
	data_offset: u32,
	#[debug(format = "{:#x}")]
	flags: u32,
}

impl Entry32 {
	fn read(mut reader: impl Read) -> Result<Self, PakReadError> {
		let mut file_name_bytes = [0; 128];
		reader.read_exact(&mut file_name_bytes)?;

		let size = reader.ioread()?;

		let mut file_key = [0; 20];
		reader.read_exact(&mut file_key)?;

		let data_offset = reader.ioread()?;
		let flags = reader.ioread()?;

		// decrypt filename
		GustPak::decrypt(&mut file_name_bytes, &file_key, None);

		// convert filename to string
		let file_name_cstr = CStr::from_bytes_until_nul(&file_name_bytes)?;
		let file_name_str = file_name_cstr.to_str()?;
		let file_name = file_name_str.to_string();
		debug_assert!(file_name.is_ascii(), "file names should be ascii");

		Ok(Self {
			file_name,
			file_size: size,
			file_key,
			data_offset,
			flags,
		})
	}
}

/// File entries for games starting from A18 (Atelier Firis)
#[derive(custom_debug::Debug, Clone)]
pub struct Entry64 {
	file_name: String,
	#[debug(format = "{:#x}")]
	file_size: u32,
	/// The xor key used to decrypt the file name and content.
	file_key: [u8; 20],
	/// The data offset of this file's data in the .pak file. This is not the file offset.
	#[debug(format = "{:#x}")]
	data_offset: u64,
	#[debug(format = "{:#x}")]
	flags: u64,
}

impl Entry64 {
	fn read(mut reader: impl Read) -> Result<Self, PakReadError> {
		// NOTE: this data type is only used for games before A22 and the pak key was only
		// introduced in A23, so we don't need to handle this.

		let mut file_name_bytes = [0; 128];
		reader.read_exact(&mut file_name_bytes)?;

		let size = reader.ioread()?;

		let mut file_key = [0; 20];
		reader.read_exact(&mut file_key)?;

		let data_offset = reader.ioread()?;
		let flags = reader.ioread()?;

		// decrypt filename
		GustPak::decrypt(&mut file_name_bytes, &file_key, None);

		// convert filename to string
		let file_name_cstr = CStr::from_bytes_until_nul(&file_name_bytes)?;
		let file_name_str = file_name_cstr.to_str()?;
		let file_name = file_name_str.to_string();
		debug_assert!(file_name.is_ascii(), "file names should be ascii");

		Ok(Self {
			file_name,
			file_size: size,
			file_key,
			data_offset,
			flags,
		})
	}
}

#[derive(custom_debug::Debug, Clone)]
/// File entries for games starting from A22 (Atelier Ryza 2)
pub struct Entry64Ext {
	file_name: String,
	#[debug(format = "{:#x}")]
	file_size: u32,
	/// The xor key used to decrypt the file name and content. This may also be xored by the pak
	/// key, if any.
	file_key: [u8; 32],
	#[debug(format = "{:#x}")]
	extra: u32,
	/// The data offset of this file's data in the .pak file. This is not the file offset.
	#[debug(format = "{:#x}")]
	data_offset: u64,
	#[debug(format = "{:#x}")]
	flags: u64,
}

impl Entry64Ext {
	fn read(mut reader: impl Read, pak_key: Option<&[u8; 32]>) -> Result<Self, PakReadError> {
		let mut file_name_bytes = [0; 128];
		reader.read_exact(&mut file_name_bytes)?;

		let file_size = reader.ioread()?;

		let mut file_key = [0; 32];
		reader.read_exact(&mut file_key)?;

		let extra = reader.ioread()?;
		let data_offset = reader.ioread()?;
		let flags = reader.ioread()?;

		// decrypt filename
		GustPak::decrypt(&mut file_name_bytes, &file_key, pak_key);

		// convert filename to string
		let file_name_cstr = CStr::from_bytes_until_nul(&file_name_bytes)?;
		let file_name_str = file_name_cstr.to_str()?;
		let file_name = file_name_str.to_string();
		debug_assert!(file_name.is_ascii(), "file names should be ascii");

		Ok(Self {
			file_name,
			file_size,
			file_key,
			extra,
			data_offset,
			flags,
		})
	}
}

/// A common representation of the file entries in a .pak file.
pub enum PakEntryList {
	Entry32(Vec<Entry32>),
	Entry64(Vec<Entry64>),
	Entry64Ext(Vec<Entry64Ext>),
}

impl PakEntryList {
	#[must_use]
	pub fn len(&self) -> usize {
		match self {
			PakEntryList::Entry32(v) => v.len(),
			PakEntryList::Entry64(v) => v.len(),
			PakEntryList::Entry64Ext(v) => v.len(),
		}
	}

	#[must_use]
	pub fn is_empty(&self) -> bool {
		self.len() == 0
	}

	/// Creates an iterator over a common representation of the entries.
	pub fn iter(&self) -> impl Iterator<Item = PakEntryRef> + '_ {
		PakEntryIterator {
			list: self,
			index: 0,
		}
	}
}

struct PakEntryIterator<'pak> {
	list: &'pak PakEntryList,
	index: usize,
}

impl<'pak> Iterator for PakEntryIterator<'pak> {
	type Item = PakEntryRef<'pak>;

	fn next(&mut self) -> Option<Self::Item> {
		match self.list {
			PakEntryList::Entry32(v) => {
				let entry = v.get(self.index)?;
				self.index += 1;
				Some(PakEntryRef::Entry32(entry))
			}
			PakEntryList::Entry64(v) => {
				let entry = v.get(self.index)?;
				self.index += 1;
				Some(PakEntryRef::Entry64(entry))
			}
			PakEntryList::Entry64Ext(v) => {
				let entry = v.get(self.index)?;
				self.index += 1;
				Some(PakEntryRef::Entry64Ext(entry))
			}
		}
	}
}

/// An owned version of [PakEntry]
#[derive(Debug, Clone)]
pub enum PakEntry {
	Entry32(Entry32),
	Entry64(Entry64),
	Entry64Ext(Entry64Ext),
}

impl PakEntry {
	pub fn as_ref(&self) -> PakEntryRef {
		match self {
			PakEntry::Entry32(e) => PakEntryRef::Entry32(e),
			PakEntry::Entry64(e) => PakEntryRef::Entry64(e),
			PakEntry::Entry64Ext(e) => PakEntryRef::Entry64Ext(e),
		}
	}
}

#[derive(Debug, Clone, Copy)]
/// A common representation of a file in a .pak file.
pub enum PakEntryRef<'pak> {
	Entry32(&'pak Entry32),
	Entry64(&'pak Entry64),
	Entry64Ext(&'pak Entry64Ext),
}

impl<'pak> PakEntryRef<'pak> {
	/// Gets the file name
	pub fn get_file_name(&'pak self) -> &'pak str {
		match self {
			PakEntryRef::Entry32(e) => &e.file_name,
			PakEntryRef::Entry64(e) => &e.file_name,
			PakEntryRef::Entry64Ext(e) => &e.file_name,
		}
	}

	/// Gets the file size
	pub fn get_file_size(&'pak self) -> u32 {
		match self {
			PakEntryRef::Entry32(e) => e.file_size,
			PakEntryRef::Entry64(e) => e.file_size,
			PakEntryRef::Entry64Ext(e) => e.file_size,
		}
	}

	/// Gets the file offset
	fn get_data_offset(&'pak self) -> u64 {
		match self {
			PakEntryRef::Entry32(e) => e.data_offset as u64,
			PakEntryRef::Entry64(e) => e.data_offset,
			PakEntryRef::Entry64Ext(e) => e.data_offset,
		}
	}

	/// Gets the file's encryption key
	fn get_file_key(&'pak self) -> &'pak [u8] {
		match self {
			PakEntryRef::Entry32(e) => &e.file_key,
			PakEntryRef::Entry64(e) => &e.file_key,
			PakEntryRef::Entry64Ext(e) => &e.file_key,
		}
	}

	/// Get a reader for the file's unencrypted data.
	pub fn get_reader<'file>(
		&'pak self,
		file: impl Read + Seek + 'file,
		pak: &'pak GustPak,
		game_version: GameVersion,
	) -> std::io::Result<impl Read + Seek + 'file> {
		let offset = pak.data_start;
		self.get_reader_with_data_start(file, offset, game_version)
	}

	pub fn get_reader_with_data_start<'file>(
		&'pak self,
		mut file: impl Read + Seek + 'file,
		data_start: u64,
		game_version: GameVersion,
	) -> std::io::Result<impl Read + Seek + 'file> {
		// default to null bytes if no pak_key was given
		let mut pak_key = GustPak::get_pak_key(game_version)
			.cloned()
			.unwrap_or_default();

		let file_key = self.get_file_key();

		debug_assert!(
			file_key.len() <= pak_key.len(),
			"file key may not be larger than pak key"
		);

		// xor pak_key with file_key to get xor key
		pak_key.iter_mut().enumerate().for_each(|(i, b)| {
			*b ^= file_key[i % file_key.len()];
		});

		let xor_key = &pak_key[..file_key.len()];
		trace!("Creating reader with xor key: {:?}", xor_key);

		file.seek(io::SeekFrom::Start(data_start + self.get_data_offset()))?;
		Ok(XorReader::new(
			FencedReader::take(file, self.get_file_size() as u64)?,
			xor_key,
		))
	}

	pub fn into_owned(self) -> PakEntry {
		match self {
			PakEntryRef::Entry32(e) => PakEntry::Entry32(e.clone()),
			PakEntryRef::Entry64(e) => PakEntry::Entry64(e.clone()),
			PakEntryRef::Entry64Ext(e) => PakEntry::Entry64Ext(e.clone()),
		}
	}
}

#[derive(Debug)]
enum PakEntryType {
	Entry32,
	Entry64,
	Entry64Ext,
}

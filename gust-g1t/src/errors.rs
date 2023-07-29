use thiserror::Error;

#[derive(Error, Debug)]
pub enum G1tReadError {
	#[error("IO error: {0}")]
	IoError(#[from] std::io::Error),
	#[error("UTF-8 error: {0}")]
	Utf8Error(#[from] std::str::Utf8Error),
	#[error("UTF-8 error: {0}")]
	FromUtf8Error(#[from] std::string::FromUtf8Error),
	#[error("C string has no null terminator: {0}")]
	CStringFromBytesUntilNullError(#[from] core::ffi::FromBytesUntilNulError),

	#[error("Invalid header magic: {0:#x} (expected 'G1TG' or 'GT1G')")]
	InvalidHeaderMagic(u32),
	#[error("Failed to parse version: {0}")]
	VersionParseError(std::num::ParseIntError),
	#[error("File size, expected {0} but found {1}")]
	InvalidTotalSize(u32, u32),
	#[error("Unknown platform: {0}")]
	UnknownPlatform(u32),
	#[error("Invalid extra size: {0:#x}")]
	InvalidExtraSize(u32),
	#[error("Invalid extended data size: {0:#x}")]
	InvalidExtendedDataSize(u32),
	#[error("Texture has no mipmaps")]
	NoMipmaps,
}

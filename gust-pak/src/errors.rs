use thiserror::Error;

#[derive(Error, Debug)]
pub enum PakReadError {
	#[error("IO error: {0}")]
	IoError(#[from] std::io::Error),
	#[error("UTF-8 error: {0}")]
	Utf8Error(#[from] std::str::Utf8Error),
	#[error("C string has no null terminator: {0}")]
	CStringFromBytesUntilNullError(#[from] core::ffi::FromBytesUntilNulError),

	#[error("Invalid header version: {0:#x} (expected 0x20000)")]
	InvalidHeaderVersion(u32),
	#[error("Invalid header size: {0} (expected 16)")]
	InvalidHeaderSize(u32),
	#[error("Too many files: {0} (max 65536)")]
	TooManyFiles(u32),
}

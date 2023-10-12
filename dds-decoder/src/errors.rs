use thiserror::Error;

use crate::DdsFormat;

#[derive(Error, Debug)]
pub enum DdsDecodeError {
	#[error("BC7 error: {0:?}")]
	Bc7Error(#[from] Bc7Error),
	#[error("Unsupported format: {0:?}")]
	UnsupportedFormat(DdsFormat),
}

#[derive(Error, Debug)]
pub enum Bc7Error {
	#[error("Invalid block mode: {0}")]
	InvalidBc7BlockMode(u8),
}

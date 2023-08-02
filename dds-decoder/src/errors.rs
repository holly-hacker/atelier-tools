use thiserror::Error;

#[derive(Error, Debug)]
pub enum DdsDecodeError {
	#[error("BC7 error: {0:?}")]
	Bc7Error(#[from] Bc7Error),
}

#[derive(Error, Debug)]
pub enum Bc7Error {
	#[error("Invalid block mode: {0}")]
	InvalidBc7BlockMode(u8),
	#[error("Unimplemented block mode: {0}")]
	UnimplementedBc7BlockMode(u8),
}

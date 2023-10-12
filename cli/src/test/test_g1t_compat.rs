use std::{
	borrow::Cow,
	ffi::OsStr,
	str::FromStr,
	sync::{
		atomic::{AtomicUsize, Ordering},
		Mutex,
	},
};

use anyhow::Context;
use argh::FromArgs;
use gust_g1t::GustG1t;
use gust_pak::GustPak;
use rayon::prelude::*;
use tracing::{debug, error, info};

/// Check how many g1t files in the game files are supported
#[derive(FromArgs)]
#[argh(subcommand, name = "g1t")]
pub struct TestG1tCompatibility {
	#[argh(positional)]
	pub input: std::path::PathBuf,

	/// the game version to use, eg. `A24` for Atelier Ryza 3
	#[argh(option, short = 'g')]
	pub game: String,
}

impl TestG1tCompatibility {
	pub fn handle(&self) -> Result<(), anyhow::Error> {
		let Ok(game_version) = gust_pak::common::GameVersion::from_str(&self.game) else {
			error!("Invalid game version: {}", self.game);
			return Ok(());
		};
		info!("Using encryption keys for {}", game_version.get_name());

		if !self.input.exists() {
			error!("input directory does not exist: {:?}", self.input);
			return Ok(());
		}

		// check if there are .PAK files in this folder (like in A17), otherwise use /Data
		let data_dir = if std::fs::read_dir(&self.input)?
			.flatten()
			.any(|f| f.path().extension() == Some(OsStr::new("PAK")))
		{
			info!("Found .PAK files in input directory");
			self.input.clone()
		} else {
			info!("No .PAK files found in input directory, reading from /Data");
			self.input.join("Data")
		};

		info!("Trying to decode all textures, this may take a few minutes");

		// TODO: this can happen in parallel
		let mut total_textures = AtomicUsize::new(0);
		let mut total_unsupported_textures = AtomicUsize::new(0);
		let stdout_lock = Mutex::new(());
		std::fs::read_dir(data_dir)?
			.collect::<std::io::Result<Vec<_>>>()?
			.into_par_iter()
			.try_for_each(|item| -> anyhow::Result<()> {
				if !item.file_type()?.is_file() {
					debug!("Skipping {:?} because it's not a file", item.path());
					return Ok(());
				}

				if item.path().extension() != Some(OsStr::new("PAK")) {
					debug!("Skipping {:?} because it's not a .PAK file", item.path());
					return Ok(());
				}

				debug!("Reading {:?}", item.path());
				let file = std::fs::File::open(item.path()).context("open file")?;
				let index = GustPak::read_index(&file, game_version).context("read index")?;

				let mut unsupported_textures: Vec<(String, Cow<'static, str>)> = vec![];
				let mut total_texture_count = 0;
				for entry in index.entries.iter() {
					let file_name = entry.get_file_name();
					if !file_name.ends_with(".g1t") {
						continue;
					}

					let span =
						tracing::trace_span!("reading g1t file", file_name = entry.get_file_name());
					_ = span.enter();

					let reader = entry.get_reader(&file, &index, game_version)?;

					let g1t = GustG1t::read(reader)
						.with_context(|| format!("read g1t file `{file_name}`"))?;

					for texture in &g1t.textures {
						total_texture_count += 1;

						let reader = entry.get_reader(&file, &index, game_version)?;
						if let Err(e) = g1t.read_image(texture, reader) {
							unsupported_textures.push((file_name.to_owned(), e.to_string().into()));
						}
					}
				}

				total_textures.fetch_add(total_texture_count, Ordering::Relaxed);
				total_unsupported_textures.fetch_add(unsupported_textures.len(), Ordering::Relaxed);

				let stdout_guard = stdout_lock.lock().expect("output lock poisoned");

				if unsupported_textures.is_empty() {
					info!(
						"{}: {} textures, all supported",
						item.path().to_string_lossy(),
						total_texture_count
					);
				} else {
					info!(
						"{}: {} textures, {} unsupported",
						item.path().to_string_lossy(),
						total_texture_count,
						unsupported_textures.len()
					);
					for (texture_name, reason) in unsupported_textures {
						info!("  {}: {}", texture_name, reason);
					}
				}

				drop(stdout_guard);

				Ok(())
			})?;

		let total_textures = *total_textures.get_mut();
		let total_unsupported_textures = *total_unsupported_textures.get_mut();

		info!(
			"{}/{} ({:.1}%) textures supported",
			(total_textures - total_unsupported_textures),
			total_textures,
			(total_textures - total_unsupported_textures) as f64 / (total_textures as f64) * 100.,
		);

		Ok(())
	}
}

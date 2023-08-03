use std::{borrow::Cow, str::FromStr};

use anyhow::Context;
use argh::FromArgs;
use gust_g1t::GustG1t;
use gust_pak::GustPak;
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

	/// test g1t compatibility by decoding the textures, rather than running static checks
	#[argh(switch, short = 'd', long = "decode")]
	pub decode: bool,
}

impl TestG1tCompatibility {
	pub fn handle(&self) -> Result<(), anyhow::Error> {
		let Ok(game_version) = gust_pak::common::GameVersion::from_str(&self.game) else {
			error!("Invalid game version: {}", self.game);
			return Ok(());
		};
		info!("Using encryption keys for {}", game_version.get_name());

		let data_dir = self.input.join("Data");
		if !data_dir.exists() {
			error!("Data directory does not exist: {:?}", data_dir);
			return Ok(());
		}

		if self.decode {
			info!("Trying to decode all textures, this may take a few minutes");
		}

		// TODO: this can happen in parallel
		let mut total_textures = 0;
		let mut total_unsupported_textures = 0;
		for item in std::fs::read_dir(&data_dir)? {
			let item = item?;
			if !item.file_type()?.is_file() {
				debug!("Skipping {:?} because it's not a file", item.path());
				continue;
			}

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

					match self.decode {
						true => {
							let reader = entry.get_reader(&file, &index, game_version)?;
							if let Err(e) = g1t.read_image(texture, reader) {
								unsupported_textures
									.push((file_name.to_owned(), e.to_string().into()));
							}
						}
						false => {
							if let Err(e) = Self::check_texture_compatible(&g1t.header, texture) {
								unsupported_textures.push((file_name.to_owned(), e));
							}
						}
					}
				}
			}

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
				total_textures += total_texture_count;
				for (texture_name, reason) in unsupported_textures {
					info!("  {}: {}", texture_name, reason);
					total_unsupported_textures += 1;
				}
			}
		}

		info!(
			"{}/{} textures unsupported",
			total_unsupported_textures, total_textures,
		);

		Ok(())
	}

	fn check_texture_compatible(
		header: &gust_g1t::G1tHeader,
		texture: &gust_g1t::TextureInfo,
	) -> Result<(), Cow<'static, str>> {
		if header.platform != gust_g1t::Platform::Windows {
			return Err(format!("Unsupported platform: {:?}", header.platform).into());
		}

		if texture.header.mipmaps > 1 {
			return Err("more than 1 mipmap".into());
		}
		if texture.header.z_mipmaps > 0 {
			return Err("z-mipmaps".into());
		}
		if !matches!(texture.header.texture_type, 0x59 | 0x5F) {
			return Err(format!(
				"Unsupported texture type: 0x{:02X}",
				texture.header.texture_type
			)
			.into());
		}

		if texture.frames > 1 {
			return Err("more than 1 frame".into());
		}

		Ok(())
	}
}

use std::{
	fs::File,
	path::{Path, PathBuf},
	str::FromStr,
};

use anyhow::Context;
use argh::FromArgs;
use gust_g1t::GustG1t;
use gust_pak::GustPak;
use tracing::{debug, error, info, trace};

/// Top-level command
#[derive(FromArgs)]
struct CliArgs {
	/// enable verbose logging
	#[argh(switch, short = 'v')]
	pub verbose: bool,

	/// enable trace logging
	#[argh(switch, short = 't')]
	pub trace: bool,

	#[argh(subcommand)]
	pub subcommand: SubCommand,
}

#[derive(FromArgs)]
#[argh(subcommand)]
enum SubCommand {
	Pak(PakSubCommand),
	G1t(G1tSubCommand),
}

/// Extract .pak files
#[derive(FromArgs)]
#[argh(subcommand, name = "pak")]
struct PakSubCommand {
	/// the input .pak file
	#[argh(positional)]
	pub input: PathBuf,

	/// the output directory
	#[argh(positional)]
	pub output: Option<PathBuf>,

	/// don't extract any files, only list them
	#[argh(switch, short = 'l')]
	pub list: bool,

	/// the game version to use, eg. `A24` for Atelier Ryza 3
	#[argh(option, short = 'g')]
	pub game: String,
}

/// Extract .g1t files
#[derive(FromArgs)]
#[argh(subcommand, name = "g1t")]
struct G1tSubCommand {
	/// the input .g1t file
	#[argh(positional)]
	pub input: PathBuf,

	/// the output directory
	#[argh(positional)]
	pub output: Option<PathBuf>,
}

fn main() {
	let args: CliArgs = argh::from_env();

	let log_level = if args.trace {
		tracing::Level::TRACE
	} else if args.verbose || cfg!(debug_assertions) {
		tracing::Level::DEBUG
	} else {
		tracing::Level::INFO
	};

	let subscriber = tracing_subscriber::fmt().with_max_level(log_level).finish();
	tracing::subscriber::set_global_default(subscriber).expect("set global tracing subscriber");

	let time_before_command_handling = std::time::Instant::now();
	let result = match args.subcommand {
		SubCommand::Pak(args) => handle_pak(args),
		SubCommand::G1t(args) => handle_g1t(args),
	};
	let time_elapsed = time_before_command_handling.elapsed();
	info!("Time elapsed: {:?}", time_elapsed);

	if let Err(e) = result {
		error!("Error: {:?}", e);
	}
}

fn handle_pak(args: PakSubCommand) -> anyhow::Result<()> {
	let Ok(game_version) = gust_pak::common::GameVersion::from_str(&args.game) else {
		error!("Invalid game version: {}", args.game);
		return Ok(());
	};
	info!("Using encryption keys for {}", game_version.get_name());

	debug!("Pak file: {:?}", args.input);

	if !args.input.is_file() {
		return Err(std::io::Error::new(
			std::io::ErrorKind::NotFound,
			"Path is not a file",
		))?;
	}
	let mut file = File::open(&args.input)?;

	let pak = GustPak::read_index(&mut file, game_version).context("read pak file")?;
	info!("Found {} files in PAK file", pak.entries.len());

	if args.list {
		for entry in pak.entries.iter() {
			println!(
				"- {} ({} bytes)",
				entry.get_file_name(),
				entry.get_file_size()
			);
		}
		return Ok(());
	}

	// start extracting the files
	let output_path = args.output.unwrap_or_else(|| {
		trace!("no output path specified, using input directory");
		args.input
			.parent()
			.expect("input path has no parent")
			.to_owned()
	});
	info!("Writing files to {}", output_path.to_string_lossy());

	for entry in pak.entries.iter() {
		let mut reader = entry
			.get_reader(&mut file, &pak, game_version)
			.context("get entry reader")?;

		let entry_path = entry
			.get_file_name()
			.replace('\\', std::path::MAIN_SEPARATOR_STR);
		let entry_path = Path::new(entry_path.trim_start_matches(std::path::MAIN_SEPARATOR_STR));

		let file_path = output_path.join(entry_path);
		let file_directory = file_path.parent().context("file path has no parent")?;
		std::fs::create_dir_all(file_directory).context("failed to create directory")?;

		let mut file = match std::fs::File::create(&file_path) {
			Ok(file) => file,
			Err(e) => {
				error!("Failed to create file: {}", e);
				continue;
			}
		};

		debug!("Writing file: {:?}", file_path);
		std::io::copy(&mut reader, &mut file).context("failed to write file")?;
	}

	Ok(())
}

fn handle_g1t(args: G1tSubCommand) -> anyhow::Result<()> {
	debug!("g1t file: {:?}", args.input);

	if !args.input.is_file() {
		return Err(std::io::Error::new(
			std::io::ErrorKind::NotFound,
			"Path is not a file",
		))?;
	}
	let mut file = File::open(&args.input)?;

	let _g1t = GustG1t::read(&mut file).context("read g1t file")?;
	info!("Read g1t file");

	let image_bytes = _g1t.read_image(&mut file).context("read image")?;
	let image_buffer = image::RgbaImage::from_vec(_g1t.width, _g1t.height, image_bytes)
		.context("image to rgbimage vec")?;

	let output_path = args.output.unwrap_or_else(|| {
		trace!("no output path specified, using input directory");
		args.input
			.parent()
			.expect("input path has no parent")
			.join("image.png")
	});
	image_buffer
		.save_with_format(output_path, image::ImageFormat::Png)
		.context("save file")?;

	Ok(())
}

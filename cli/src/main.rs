mod test;

use std::{
	fs::File,
	path::{Path, PathBuf},
	str::FromStr,
};

use anyhow::Context;
use argh::FromArgs;
use gust_g1t::GustG1t;
use gust_pak::GustPak;
use test::TestSubCommand;
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
	Test(TestSubCommand),
}

/// Extract .pak files
#[derive(FromArgs)]
#[argh(subcommand, name = "pak")]
struct PakSubCommand {
	/// the input .pak file or a directory containing .pak files
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
	/// the input .g1t file, or a directory containing .g1t files
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
		SubCommand::Test(args) => args.handle(),
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

	let input_files = if args.input.is_dir() {
		let mut input_files = Vec::new();
		for entry in std::fs::read_dir(&args.input)? {
			let entry = entry?;
			let path = entry.path();
			if path.is_file()
				&& path
					.extension()
					.map_or(false, |ext| ext.to_ascii_lowercase() == "pak")
			{
				input_files.push(path);
			}
		}
		info!("Found {} PAK files", input_files.len());
		input_files
	} else if args.input.is_file() {
		vec![args.input.clone()]
	} else {
		Err(std::io::Error::new(
			std::io::ErrorKind::NotFound,
			"Path is not a file or directory",
		))?
	};

	for input in input_files {
		let mut file = File::open(&input)?;

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
		let output_path = args.output.clone().unwrap_or_else(|| {
			trace!("no output path specified, using input directory");
			input.parent().expect("input path has no parent").to_owned()
		});
		info!("Writing files to {}", output_path.to_string_lossy());

		for entry in pak.entries.iter() {
			let mut reader = entry
				.get_reader(&mut file, &pak, game_version)
				.context("get entry reader")?;

			let entry_path = entry
				.get_file_name()
				.replace('\\', std::path::MAIN_SEPARATOR_STR);
			let entry_path =
				Path::new(entry_path.trim_start_matches(std::path::MAIN_SEPARATOR_STR));

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
	}

	Ok(())
}

fn handle_g1t(args: G1tSubCommand) -> anyhow::Result<()> {
	debug!("g1t file: {:?}", args.input);

	let input_files = if args.input.is_dir() {
		let mut input_files = Vec::new();
		for entry in std::fs::read_dir(&args.input)? {
			let entry = entry?;
			let path = entry.path();
			if path.is_file()
				&& path
					.extension()
					.map_or(false, |ext| ext.to_ascii_lowercase() == "g1t")
			{
				input_files.push(path);
			}
		}
		info!("Found {} g1t files", input_files.len());
		input_files
	} else if args.input.is_file() {
		vec![args.input.clone()]
	} else {
		Err(std::io::Error::new(
			std::io::ErrorKind::NotFound,
			"Path is not a file or directory",
		))?
	};

	for input in input_files {
		let mut file = File::open(&input)?;

		debug!("reading g1t file...");
		let g1t = GustG1t::read(&mut file).context("read g1t file")?;
		info!("Read g1t file");

		let texture_count = g1t.textures.len();

		if texture_count == 0 {
			info!("No textures found");
			continue;
		}

		for (texture_index, texture) in g1t.textures.iter().enumerate() {
			let image_bytes = g1t.read_image(texture, &mut file).context("read image")?;
			let image_buffer =
				image::RgbaImage::from_vec(texture.width, texture.height, image_bytes)
					.context("image to rgbimage vec")?;

			let output_dir = args.output.clone().unwrap_or_else(|| {
				trace!("no output directory specified, using input directory");
				input
					.parent()
					.expect("input path has no parent")
					.to_path_buf()
			});

			let texture_idx_string = (g1t.textures.len() > 1)
				.then(|| format!("_{texture_index}"))
				.unwrap_or_default();
			let output_file_name = input
				.file_stem()
				.expect("get file stem")
				.to_str()
				.expect("file name to string")
				.to_owned() + texture_idx_string.as_str()
				+ ".png";

			let output_path = output_dir.join(output_file_name);

			debug!("saving image...");
			image_buffer
				.save_with_format(output_path, image::ImageFormat::Png)
				.context("save file")?;
			info!("Image saved");
		}
	}

	Ok(())
}

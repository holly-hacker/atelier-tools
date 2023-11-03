use strum::EnumMessage;

/// An enum of supported games.
///
/// If a game has some associated data that is specific to only 1 package, it should be defined in
/// that package.
#[derive(
	Debug, Copy, Clone, PartialEq, Eq, strum::EnumMessage, strum::EnumString, strum::EnumIter,
)]
#[strum(ascii_case_insensitive)]
pub enum GameVersion {
	/// Atelier Sophie: The Alchemist of the Mysterious Book
	#[strum(message = "Atelier Sophie")]
	#[strum(detailed_message = "Atelier Sophie: The Alchemist of the Mysterious Book")]
	A17,
	/// Atelier Firis: The Alchemist and the Mysterious Journey
	#[strum(message = "Atelier Firis")]
	#[strum(detailed_message = "Atelier Firis: The Alchemist and the Mysterious Journey")]
	A18,
	/// Atelier Lydie & Suelle: The Alchemists and the Mysterious Paintings
	#[strum(message = "Atelier Lydie & Suelle")]
	#[strum(
		detailed_message = "Atelier Lydie & Suelle: The Alchemists and the Mysterious Paintings"
	)]
	A19,
	/// Atelier Ryza: Ever Darkness & the Secret Hideout
	#[strum(message = "Atelier Ryza")]
	#[strum(detailed_message = "Atelier Ryza: Ever Darkness & the Secret Hideout")]
	A21,
	/// Atelier Ryza 2: Lost Legends & the Secret Fairy
	#[strum(message = "Atelier Ryza 2")]
	#[strum(detailed_message = "Atelier Ryza 2: Lost Legends & the Secret Fairy")]
	A22,
	/// Atelier Sophie 2: The Alchemist of the Mysterious Dream
	#[strum(message = "Atelier Sophie 2")]
	#[strum(detailed_message = "Atelier Sophie 2: The Alchemist of the Mysterious Dream")]
	A23,
	/// Atelier Ryza 3: Alchemist of the End & the Secret Key
	#[strum(message = "Atelier Ryza 3")]
	#[strum(detailed_message = "Atelier Ryza 3: Alchemist of the End & the Secret Key")]
	A24,
}

impl GameVersion {
	/// Returns the short name of the game, eg. `Atelier Ryza 3`
	pub fn get_short_name(self) -> &'static str {
		self.get_message().expect("game should have game name")
	}

	/// Returns the full name of the game, eg. `Atelier Ryza 3: Alchemist of the End & the Secret
	/// Key`
	pub fn get_name(self) -> &'static str {
		self.get_detailed_message()
			.or_else(|| self.get_message())
			.expect("game should have game name")
	}
}

#[cfg(test)]
mod tests {
	use strum::IntoEnumIterator;

	#[test]
	fn each_game_has_name() {
		for game in super::GameVersion::iter() {
			// run to ensure it does not panic
			_ = game.get_short_name();
			_ = game.get_name();
		}
	}
}

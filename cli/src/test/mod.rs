mod test_g1t_compat;

use argh::FromArgs;

/// Run some test commands
#[derive(FromArgs)]
#[argh(subcommand, name = "test")]
pub struct TestSubCommand {
	#[argh(subcommand)]
	pub subcommand: TestSubCommandEnum,
}

impl TestSubCommand {
	pub fn handle(self) -> anyhow::Result<()> {
		match self.subcommand {
			TestSubCommandEnum::G1t(args) => args.handle(),
		}
	}
}

#[derive(FromArgs)]
#[argh(subcommand)]
pub enum TestSubCommandEnum {
	G1t(test_g1t_compat::TestG1tCompatibility),
}

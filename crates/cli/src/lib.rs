mod commands;
mod helpers;

use clap::{ArgAction, Parser};

use commands::Commands;
use commands::init::{cmd_init, CmdInitOptions};
use commands::build::{cmd_build, CmdBuildOptions};

const ABOUT: &str = "Welcome to Ruxy CLI! Use it to initialize, build, and run your project.";

#[derive(Parser)]
#[command(
  version,
  about = ABOUT,
  long_about = None,
  subcommand_required = true
)]
struct Cli {
  #[arg(short, long, action = ArgAction::SetTrue)]
  debug: bool,

  #[command(subcommand)]
  command: Commands,
}

pub fn cli() {
  let cli = Cli::parse();

  if cli.debug {
    println!("Debug mode is on");
  }

  match cli.command {
    Commands::Init { directory, name, enable_typescript } => {
      cmd_init(CmdInitOptions { directory, name, enable_typescript });
    }
    Commands::Build { directory } => {
      cmd_build(CmdBuildOptions { directory })
    }
    Commands::Dev { .. } => {}
    Commands::Serve { .. } => {}
  }
}

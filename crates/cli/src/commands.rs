pub mod build;
pub mod init;

use std::path::PathBuf;

use clap::Subcommand;

#[derive(Subcommand)]
pub enum Commands {
  #[command(about = "Initialize a new Ruxy project")]
  Init {
    #[arg(
      short,
      long,
      alias = "dir",
      help = "A relative or absolute path to an existing directory to initialize your project in"
    )]
    directory: Option<PathBuf>,
    #[arg(short, long, help = "The name of your project")]
    name: Option<String>,
    #[arg(long, help = "Enable TypeScript support in your client code")]
    enable_typescript: Option<bool>,
  },
  #[command(about = "Create a production build of your project")]
  Build {
    #[arg(short, long, alias = "dir", help = "A relative or absolute path to your project's directory")]
    directory: Option<PathBuf>,
  },
  #[command(about = "Run your project in development mode")]
  Dev {
    #[arg(short, long, alias = "dir", help = "A relative or absolute path to your project's directory")]
    directory: Option<PathBuf>,
    #[arg(short, long)]
    port: Option<usize>,
  },
  #[command(about = "Serve your production build")]
  Serve {
    #[arg(short, long)]
    port: Option<usize>,
  },
}

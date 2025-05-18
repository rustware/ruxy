use std::path::{Path, PathBuf};

pub fn resolve_project_directory(dir_arg: Option<PathBuf>) -> PathBuf {
  let Some(dir) = dir_arg else {
    return get_cwd_as_project_dir();
  };

  if !is_project_dir(&dir) {
    eprintln!(
      "The path specified with the --dir flag is not a Ruxy project directory.\
      Please either run this command in your project directory or specify a project directory with the --dir flag."
    );

    std::process::exit(1);
  }

  dir
}

fn get_cwd_as_project_dir() -> PathBuf {
  let Ok(cwd) = std::env::current_dir() else {
    eprintln!("Failed to get current working directory");
    std::process::exit(1);
  };

  if !is_project_dir(&cwd) {
    eprintln!(
      "This is not a Ruxy project directory.\
      Please either run this command in your project directory or specify a project directory with the --dir flag."
    );

    std::process::exit(1);
  }

  cwd
}

fn is_project_dir(dir: &Path) -> bool {
  // We only check for the existence of the ruxy.toml and src/app.rs files.
  // This is not a perfect check, but it's good enough for the CLI to make
  // decisions.
  dir.join("ruxy.toml").is_file() && dir.join("src/app.rs").is_file()
}

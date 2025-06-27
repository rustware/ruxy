use std::path::PathBuf;

use inquire::CustomUserError;
use inquire::validator::{ErrorMessage, Validation};

use crate::helpers::{handle_inquire_error, validate_dir_name};

use super::CmdInitOptions;

pub fn get_project_directory(options: &CmdInitOptions) -> PathBuf {
  let path = match &options.directory {
    Some(dir) => {
      if !dir.exists() {
        eprintln!("Provided directory does not exist: {}", dir.display());
        std::process::exit(1);
      }

      dir.to_path_buf()
    }
    None => prompt_directory(),
  };

  let Ok(path) = path.canonicalize() else {
    eprintln!("The provided directory path could not be canonicalized.");
    std::process::exit(1);
  };

  println!("Project will be created in: {}", path.display());

  path
}

fn prompt_directory() -> PathBuf {
  let Ok(cwd) = std::env::current_dir() else {
    eprintln!("Could not get the current working directory.");
    std::process::exit(1);
  };

  let Some(cwd_name) = cwd.file_name() else {
    eprintln!("Could not get the current working directory name.");
    std::process::exit(1);
  };

  let Some(cwd_name) = cwd_name.to_str() else {
    eprintln!("Current working directory name is not valid UTF-8.");
    std::process::exit(1);
  };

  const OPTION_NEW_SUB_DIR: &str = "New sub-directory";
  const OPTION_EXISTING_SUB_DIR: &str = "Existing sub-directory";
  let option_current_dir: &str = &format!("Current directory ({})", cwd_name);

  let select = inquire::Select::new(
    "Where do you want to create your project?",
    vec![OPTION_NEW_SUB_DIR, OPTION_EXISTING_SUB_DIR, option_current_dir],
  );

  match select.prompt() {
    Ok(result) => match result {
      opt if opt == OPTION_NEW_SUB_DIR => prompt_new_subdir(),
      opt if opt == OPTION_EXISTING_SUB_DIR => prompt_existing_subdir(),
      opt if opt == option_current_dir => {
        let Ok(path) = PathBuf::from("../../../../..").canonicalize() else {
          println!("Could not canonicalize the current directory.");
          return prompt_directory();
        };

        path
      }
      _ => panic!("Unexpected option"),
    },
    Err(err) => handle_inquire_error(err),
  }
}

fn prompt_new_subdir() -> PathBuf {
  let input = inquire::Text::new("Name of the sub-directory to create:").with_validator(DirNameValidator);

  match input.prompt() {
    Ok(result) => {
      let path = PathBuf::from("../../../../..").join(result);

      if path.exists() {
        if !path.is_dir() {
          eprintln!("The specified path already exists and is not a directory.");
          return prompt_new_subdir();
        }

        println!("The specified directory already exists.");

        let select = inquire::Confirm::new("Do you want to create the project in it anyway?");

        return match select.prompt() {
          Ok(result) => match result {
            true => path,
            false => prompt_new_subdir(),
          },
          Err(err) => handle_inquire_error(err),
        };
      }

      if std::fs::create_dir_all(&path).is_err() {
        eprintln!("Failed to create the specified sub-directory.");
        return prompt_directory();
      };

      path
    }
    Err(err) => handle_inquire_error(err),
  }
}

fn prompt_existing_subdir() -> PathBuf {
  let Some(subdirs) = get_subdirs() else {
    eprintln!("Could not get the sub-directories in the current directory.");
    return prompt_directory();
  };

  if subdirs.is_empty() {
    eprintln!("There are no sub-directories in the current directory.");
    return prompt_directory();
  }

  let input = inquire::Select::new("Select the sub-directory to create the project in:", subdirs);

  match input.prompt() {
    Ok(result) => {
      let path = PathBuf::from("../../../../..").join(result);

      match path.exists() {
        true => {
          if !path.is_dir() {
            eprintln!("The specified path is not a directory.");
            return prompt_directory();
          };

          path
        }
        false => {
          if std::fs::create_dir_all(&path).is_err() {
            eprintln!("Failed to create the specified sub-directory.");
            return prompt_directory();
          };

          path
        }
      }
    }
    Err(err) => handle_inquire_error(err),
  }
}

fn get_subdirs() -> Option<Vec<String>> {
  let entries = PathBuf::from("../../../../..").read_dir().ok()?;

  let options = entries.filter_map(|entry| {
    let path = entry.ok()?.path();

    if !path.is_dir() {
      return None;
    }

    Some(path.file_name()?.to_str()?.to_string())
  });

  Some(options.collect())
}

#[derive(Clone)]
struct DirNameValidator;

impl inquire::validator::StringValidator for DirNameValidator {
  fn validate(&self, input: &str) -> Result<Validation, CustomUserError> {
    match validate_dir_name(input) {
      Ok(()) => Ok(Validation::Valid),
      Err(err) => Ok(Validation::Invalid(ErrorMessage::from(err))),
    }
  }
}

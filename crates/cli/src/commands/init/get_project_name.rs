use std::ffi::OsStr;
use std::path::Path;

use inquire::CustomUserError;
use inquire::validator::{ErrorMessage, Validation};

use crate::helpers::handle_inquire_error;

use super::CmdInitOptions;

pub fn get_project_name(options: &mut CmdInitOptions, directory: &Path) -> String {
  match &mut options.name {
    Some(name) => match validate_project_name(name) {
      Ok(_) => std::mem::take(name),
      Err(err) => {
        eprintln!("Invalid project name: {err}");
        std::process::exit(1);
      }
    },
    None => prompt_name(directory),
  }
}

fn prompt_name(directory: &Path) -> String {
  let mut input = inquire::Text::new("Enter your project name:").with_validator(ProjectNameValidator);

  if let Some(dir_name) = directory.file_name().and_then(OsStr::to_str)
    && validate_project_name(dir_name).is_ok()
  {
    input = input.with_default(dir_name);
  }

  match input.prompt() {
    Ok(name) => name,
    Err(err) => handle_inquire_error(err),
  }
}

fn validate_project_name(name: &str) -> Result<(), &str> {
  // In addition to basic validation (non-empty, ...), we also require the project name to comply with
  // NPM package name requirements (https://docs.npmjs.com/cli/v11/configuring-npm/package-json#name).

  if name.is_empty() {
    return Err("Project name cannot be empty");
  }

  if !name.chars().all(|c| c.is_ascii_lowercase() || matches!(c, '0'..='9' | '-' | '_' | '.')) {
    return Err("Project name can only contain ASCII lowercase characters, numbers, dashes, underscores, and dots");
  }

  if name.len() > 214 {
    return Err("Project name can have at most 214 characters");
  }

  if name.starts_with('.') || name.starts_with('_') {
    return Err("Project name cannot start with a dot or an underscore");
  }

  Ok(())
}

#[derive(Clone)]
struct ProjectNameValidator;

impl inquire::validator::StringValidator for ProjectNameValidator {
  fn validate(&self, input: &str) -> Result<Validation, CustomUserError> {
    match validate_project_name(input) {
      Ok(()) => Ok(Validation::Valid),
      Err(err) => Ok(Validation::Invalid(ErrorMessage::from(err))),
    }
  }
}

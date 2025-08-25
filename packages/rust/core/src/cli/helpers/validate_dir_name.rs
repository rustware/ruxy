pub fn validate_dir_name(name: &str) -> Result<(), &str> {
  if name.is_empty() {
    return Err("Directory name can't be empty");
  }

  if name.len() > 255 {
    return Err("Directory name can't be longer than 255 characters");
  }

  // Cross-platform allowed characters
  if !name.chars().all(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '.' | ' ' | '(' | ')' | '[' | ']')) {
    return Err(
      "Directory name can only contain ASCII alphanumeric characters, dashes, underscores, dots, spaces, parentheses, brackets, and slashes",
    );
  }

  // Disallow Windows reserved names (case-insensitive)
  const RESERVED: &[&str] = &[
    "CON", "PRN", "AUX", "NUL", "COM1", "COM2", "COM3", "COM4", "COM5", "COM6", "COM7", "COM8", "COM9", "LPT1", "LPT2",
    "LPT3", "LPT4", "LPT5", "LPT6", "LPT7", "LPT8", "LPT9",
  ];

  let upper = name.to_uppercase();

  if RESERVED.contains(&upper.as_str()) {
    return Err("Directory name can't be one of the Windows reserved names");
  }

  // Names can't end in dot or space on Windows
  if name.ends_with('.') || name.ends_with(' ') {
    return Err("Directory name can't end with a dot or space");
  }

  Ok(())
}

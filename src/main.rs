use colored::Colorize;
use expand_str::expand_string_with_env;
use promptuity::prompts::{Select, SelectOption};
use promptuity::themes::FancyTheme;
use promptuity::{Promptuity, Term};
use std::collections::HashSet;
use std::error::Error;
use std::path::PathBuf;
use winreg::enums::{HKEY_CURRENT_USER, HKEY_LOCAL_MACHINE, KEY_READ, KEY_WRITE};
use winreg::RegKey;

#[derive(Debug)]
enum Mode {
  User,
  System,
}

fn deduplicate_paths(paths: Vec<PathBuf>, removed_paths: &mut Vec<PathBuf>) -> Vec<PathBuf> {
  let mut unique_paths: HashSet<PathBuf> = HashSet::new();
  let mut result: Vec<PathBuf> = Vec::new();

  for path in paths {
    let original_path = path.clone();
    let canonicalized_path_option = match expand_string_with_env(path.to_str().unwrap()) {
      Ok(path) => Some(PathBuf::from(path).canonicalize().unwrap()),
      Err(err) => {
        println!("invalid path: {:?}", err);
        None
      }
    };

    if canonicalized_path_option.is_none() {
      println!(
        "{}",
        format!("├ Invalid path extension: removing {:?}", original_path).bright_black()
      );
      continue;
    }

    let canonicalized_path = canonicalized_path_option.unwrap();

    if !unique_paths.contains(&canonicalized_path.clone()) {
      unique_paths.insert(canonicalized_path);
      result.push(original_path);
    } else {
      println!(
        "{}",
        format!(
          "├ Removing duplicate path: {}",
          &original_path.to_str().unwrap()
        )
        .bright_black()
      );
      removed_paths.push(original_path);
    }
  }

  result
}

#[cfg(windows)]
fn clean(mode: Mode) -> Result<(), Box<dyn Error>> {
  use std::{env, fs};

  let root = RegKey::predef(match mode {
    Mode::User => HKEY_CURRENT_USER,
    Mode::System => HKEY_LOCAL_MACHINE,
  });
  let environment_variables = root.open_subkey_with_flags(
    match mode {
      Mode::User => r#"Environment"#,
      Mode::System => r#"SYSTEM\CurrentControlSet\Control\Session Manager\Environment"#,
    },
    KEY_READ | KEY_WRITE,
  )?;
  let current_path: String = environment_variables.get_value("PATH")?;
  let mut removed_paths = vec![];
  let paths = deduplicate_paths(
    current_path
      .split(';')
      .filter(|p| !p.is_empty())
      .map(|path_str| PathBuf::from(path_str))
      .filter(|path| {
        let expanded = PathBuf::from(expand_string_with_env(&path.to_str().unwrap()).unwrap());
        if expanded.exists() && expanded.is_dir() {
          true
        } else {
          println!(
            "{}",
            format!("├ Removing invalid path: {}", path.to_str().unwrap()).bright_black()
          );
          removed_paths.push(path.clone());
          false
        }
      })
      .collect::<Vec<PathBuf>>(),
    &mut removed_paths,
  );

  let updated_path = paths
    .iter()
    .map(|path| path.to_str().unwrap())
    .collect::<Vec<&str>>()
    .join(";");

  let removed_paths_file = env::temp_dir().join(format!("removed_paths_{:?}.txt", mode));
  if removed_paths_file.exists() {
    fs::remove_file(&removed_paths_file)?;
  }
  fs::write(
    removed_paths_file,
    removed_paths
      .iter()
      .map(|path| path.to_str().unwrap())
      .collect::<Vec<&str>>()
      .join("\n"),
  )?;

  environment_variables.set_value("PATH", &updated_path)?;

  Ok(())
}

#[cfg(not(windows))]
fn clean(_mode: Mode) -> Result<(), Box<dyn Error>> {
  unimplemented!("only windows is supported for now")
}

fn main() -> Result<(), Box<dyn Error>> {
  let mut theme = FancyTheme::default();
  let mut term = Term::default();
  let mut p = Promptuity::new(&mut term, &mut theme);

  p.term().clear()?;
  p.with_intro("Path Cleaner").begin()?;

  let mode = p.prompt(
    Select::new(
      "Select the scope of the cleaning operation",
      vec![
        SelectOption::new("User", "user"),
        SelectOption::new("System*", "system"),
        SelectOption::new("Both*", "both"),
      ],
    )
    .with_hint("*requires elevation")
  )?;

  if mode == "both" {
    println!("{}", "├ Cleaning for user".bright_black());
    clean(Mode::User)?;
    println!("{}", "◇ Done.".green());
    println!("{}", "├ Cleaning for system".bright_black());
    clean(Mode::System)?;
    println!("{}", "◇ Done.".green());
  } else {
    println!(
      "{}",
      format!("├ Cleaning for {}", mode).bright_black()
    );
    clean(match mode {
      "user" => Mode::User,
      "system" => Mode::System,
      _ => unreachable!(),
    })?;
    println!("{}", "◇ Done.".green());
  }
  println!("{}", "│".bright_black());

  p.with_outro("Finished".bold().green()).finish()?;

  Ok(())
}

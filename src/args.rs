use std::str::FromStr;

use clap::Parser;

use crate::grammars::Language;

#[derive(Debug, Parser)]
#[command(author, version, about, long_about = None)]
pub struct Args {
  #[arg(short, long)]
  language: Language,

  selections: Vec<Selection>,
}

/// Selection from `start` up to and including `end`.
#[derive(Clone, Copy, Debug)]
pub struct Selection {
  pub start: Point,
  pub end: Point,
}

impl Selection {
  const SEPARATOR: char = ',';
}

impl FromStr for Selection {
  type Err = String;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    let parts = s.split(Self::SEPARATOR).collect::<Vec<&str>>();
    let [start, end] = parts.as_slice() else {
      Err("missing selection separator")?
    };

    Ok(Self {
      start: start.parse().map_err(|err| format!("invalid start: {err:?}"))?,
      end: end.parse().map_err(|err| format!("invalid end: {err:?}"))?,
    })
  }
}

/// A 0-indexed line and column cursor position.
#[derive(Clone, Copy, Debug)]
pub struct Point {
  pub line: u64,
  pub column: u64,
}

impl Point {
  const SEPARATOR: char = '.';
}

impl FromStr for Point {
  type Err = String;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    let parts = s.split(Self::SEPARATOR).collect::<Vec<&str>>();
    let [line, column] = parts.as_slice() else {
      Err("missing point separator")?
    };

    Ok(Self {
      line: line.parse().map_err(|err| format!("invalid line: {err:?}"))?,
      column: column.parse().map_err(|err| format!("invalid column: {err:?}"))?,
    })
  }
}

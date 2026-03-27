use std::{fmt::Display, str::FromStr};

use clap::Parser;
use tree_sitter::Node;

use crate::Language;

#[derive(Debug, Parser)]
#[command(author, version, about, long_about = None)]
pub struct Args {
  #[arg(short, long)]
  pub language: Language,
  pub selections: Vec<Selection>,
}

/// Selection from `start` up to and including `end`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Selection {
  pub start: Point,
  pub end: Point,
}

impl Selection {
  const SEPARATOR: char = ',';

  pub fn new(start: Point, end: Point) -> Self {
    Self { start, end }
  }

  pub fn is_inverted(self) -> bool {
    self.start > self.end
  }

  pub fn normalized(self) -> Self {
    if self.is_inverted() { self.inverted() } else { self }
  }

  pub fn inverted(self) -> Self {
    Self {
      start: self.end,
      end: self.start,
    }
  }

  pub fn contains(self, other: Self) -> bool {
    self.start <= other.start && other.end <= self.end
  }
}

impl Display for Selection {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{}{}{}", self.start, Self::SEPARATOR, self.end)
  }
}

impl FromStr for Selection {
  type Err = String;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    let parts = s.split(Self::SEPARATOR).collect::<Vec<&str>>();
    let [start, end] = parts.as_slice() else {
      Err(format!("missing selection separator ({})", Self::SEPARATOR))?
    };

    Ok(Self {
      start: start.parse().map_err(|err| format!("invalid start: {err:?}"))?,
      end: end.parse().map_err(|err| format!("invalid end: {err:?}"))?,
    })
  }
}

impl<'a> From<&'a Node<'_>> for Selection {
  fn from(node: &'a Node) -> Self {
    let start = Point::from(node.start_position());
    let end = Point::from(node.end_position()).prev();
    Self::new(start, end)
  }
}

/// A 1-indexed line and column cursor position.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Point {
  pub line: usize,
  /// If `column == i32::MIN`, this is the point at the end of the line.
  pub column: usize,
}

impl Point {
  const SEPARATOR: char = '.';

  fn prev(self) -> Self {
    if self.column == 1 {
      Self {
        line: self.line - 1,
        // a hack specific to kakoune, as it uses i32 to store columns, and
        // i32::MIN is the end of the line.
        column: (i32::MIN as usize),
      }
    } else {
      Self {
        line: self.line,
        column: self.column - 1,
      }
    }
  }
}

impl From<Point> for tree_sitter::Point {
  fn from(point: Point) -> tree_sitter::Point {
    tree_sitter::Point {
      row: point.line - 1,
      column: point.column - 1,
    }
  }
}

impl From<tree_sitter::Point> for Point {
  fn from(tspoint: tree_sitter::Point) -> Point {
    Point {
      line: tspoint.row + 1,
      column: tspoint.column + 1,
    }
  }
}

impl Display for Point {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{}{}{}", self.line, Self::SEPARATOR, self.column)
  }
}

impl FromStr for Point {
  type Err = String;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    let parts = s.split(Self::SEPARATOR).collect::<Vec<&str>>();
    let [line, column] = parts.as_slice() else {
      Err(format!("missing point separator ({})", Self::SEPARATOR))?
    };

    Ok(Self {
      line: line.parse().map_err(|err| format!("invalid line: {err:?}"))?,
      column: column.parse().map_err(|err| format!("invalid column: {err:?}"))?,
    })
  }
}

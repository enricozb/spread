use std::{io::Read, sync::OnceLock};

use clap::{Parser as _, ValueEnum};
use tree_sitter::Parser;

use crate::args::{Args, Selection};

include!(concat!(env!("OUT_DIR"), "/", "grammars.rs"));

mod args;

languages! {
  #[allow(non_camel_case_types)]
  #[derive(Clone, Copy, Debug, ValueEnum)]
  #[value(rename_all = "lower")]
  pub enum Language
}

pub static INPUT_LINE_LENGTHS: OnceLock<Vec<usize>> = OnceLock::new();

fn main() {
  let args = Args::parse();

  let mut text = Vec::new();
  std::io::stdin().read_to_end(&mut text).unwrap();

  let line_lengths = text.split(|&b| b == b'\n').map(|l| l.len()).collect();
  INPUT_LINE_LENGTHS.set(line_lengths).unwrap();

  let mut parser = Parser::new();
  parser.set_language(&args.language.as_tree_sitter_language()).unwrap();

  let tree = parser.parse(&text, None).unwrap();
  let root = tree.root_node();

  let mut selections = Vec::new();

  for selection in args.selections {
    let inverted = selection.is_inverted();
    let selection = selection.normalized();

    let mut node = root
      .descendant_for_point_range(selection.start.into(), selection.end.into())
      .unwrap();
    let mut new_selection = Selection::from(&node);
    while selection.contains(new_selection) {
      let Some(parent) = node.parent() else { break };
      new_selection = Selection::from(&parent);
      node = parent;
    }
    if inverted {
      new_selection = new_selection.inverted();
    }

    selections.push(new_selection);
  }

  println!(
    "{}",
    selections
      .into_iter()
      .map(|s| s.to_string())
      .collect::<Vec<_>>()
      .join(" ")
  )
}

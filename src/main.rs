use std::io::Read;

use clap::Parser as _;
use tree_sitter::Parser;

use crate::args::{Args, Selection};

mod args;
include!(concat!(env!("OUT_DIR"), "/", "grammars.rs"));

fn main() {
  let args = Args::parse();

  let mut text = Vec::new();
  std::io::stdin().read_to_end(&mut text).unwrap();

  let mut parser = Parser::new();
  parser.set_language(&args.language.as_tree_sitter_language()).unwrap();

  let tree = parser.parse(&text, None).unwrap();
  let root = tree.root_node();

  let mut selections = Vec::new();

  for selection in args.selections {
    let inverted = selection.is_inverted();
    let selection = selection.normalized();

    let node = root
      .descendant_for_point_range(selection.start.into(), selection.end.into())
      .unwrap();
    let mut new_selection = Selection::from(&node);
    if selection == new_selection {
      new_selection = Selection::from(&node.parent().unwrap());
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

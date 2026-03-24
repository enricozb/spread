use clap::Parser;

use crate::args::Args;

mod args;
include!(concat!(env!("OUT_DIR"), "/", "grammars.rs"));

fn main() {
  let args = Args::parse();

  println!("{args:?}");
}

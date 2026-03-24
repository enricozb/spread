use std::{
  env,
  path::{Path, PathBuf},
};

use anyhow::{Context, Result};
use serde::Deserialize;

#[derive(Deserialize)]
struct TreeSitterConfig {
  grammars: Vec<Grammar>,
}

#[derive(Clone, Deserialize)]
struct Grammar {
  name: String,
  path: Option<PathBuf>,
  camelcase: Option<String>,
}

fn tree_sitter_json(mut grammar_path: &Path) -> Result<String> {
  let mut tree_sitter_json_path = grammar_path.join("tree-sitter.json");
  while !tree_sitter_json_path.exists() {
    grammar_path = grammar_path.parent().context("no parent")?;
    tree_sitter_json_path = grammar_path.join("tree-sitter.json");
  }
  Ok(std::fs::read_to_string(tree_sitter_json_path)?)
}

fn main() {
  println!("cargo:rerun-if-env-changed=GRAMMARS");

  let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
  let mut mods = Vec::new();

  let mut grammars = Vec::new();
  let grammar_paths = env::var("GRAMMARS").unwrap();
  for grammar_path in grammar_paths.split(":").map(PathBuf::from) {
    let tree_sitter_json = tree_sitter_json(&grammar_path).unwrap();

    let tree_sitter_config: TreeSitterConfig = serde_json::from_str(&tree_sitter_json).unwrap();
    for grammar in tree_sitter_config.grammars {
      let Grammar { path, name, .. } = &grammar;
      let grammar_path = match path {
        Some(path) => grammar_path.join(path),
        None => grammar_path.clone(),
      };

      let mut build = cc::Build::new();
      let src_path = grammar_path.join("src");
      let parser_path = src_path.join("parser.c");
      let scanner_path = src_path.join("scanner.c");
      build
        .include(src_path)
        .opt_level(2) // To ignore FORTIFY_SOURCE warnings
        .flag("-Wno-unused-but-set-variable")
        .flag("-Wno-unused-label")
        .flag("-Wno-unused-parameter")
        .flag("-Wno-unused-value")
        .file(parser_path);
      if scanner_path.exists() {
        build.file(scanner_path);
      }
      build.compile(&format!("tree-sitter-{name}"));

      mods.push(format!(
        r#"
pub mod {name} {{
  unsafe extern "C" {{ fn tree_sitter_{name}() -> tree_sitter::Language; }}

  pub fn language() -> tree_sitter::Language {{ unsafe {{ tree_sitter_{name}() }} }}
}}
        "#,
      ));
      grammars.push(grammar.clone());
    }
  }

  let mods = mods.join("\n");
  let languages = grammars
    .iter()
    .map(|g| g.camelcase.as_ref().unwrap_or(&g.name).as_str())
    .collect::<Vec<_>>()
    .join(",\n");
  let matches = grammars
    .iter()
    .map(|g| {
      format!(
        "Language::{} => {}::language()",
        g.camelcase.as_ref().unwrap_or(&g.name),
        g.name
      )
    })
    .collect::<Vec<_>>()
    .join(",\n");
  let grammars_mod = format!(
    "
pub mod grammars {{
  #[derive(Clone, Copy, Debug, clap::ValueEnum)]
  #[allow(non_camel_case_types)]
  pub enum Language {{
    {languages}
  }}

  impl Language {{
    pub fn as_tree_sitter_language(self) -> tree_sitter::Language {{
      match self {{
        {matches}
      }}
    }}
  }}

  {mods}
}}"
  );

  std::fs::write(out_dir.join("grammars.rs"), grammars_mod).unwrap();
}

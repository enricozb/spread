use std::{env, path::PathBuf};

use serde::Deserialize;

#[derive(Deserialize)]
struct TreeSitterConfig {
  grammars: Vec<Grammar>,
}

#[derive(Deserialize)]
struct Grammar {
  name: String,
  camelcase: String,
  #[serde(rename = "file-types")]
  extensions: Vec<String>,
}

fn main() {
  println!("cargo:rerun-if-env-changed=GRAMMARS");

  let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
  let mut mods = Vec::new();

  let mut languages = Vec::new();
  let grammars = env::var("GRAMMARS").unwrap();
  for grammar_path in grammars.split(":").map(PathBuf::from) {
    let tree_sitter_json_path = grammar_path.join("tree-sitter.json");
    let tree_sitter_json = std::fs::read_to_string(tree_sitter_json_path).unwrap();

    let tree_sitter_config: TreeSitterConfig = serde_json::from_str(&tree_sitter_json).unwrap();
    let [Grammar { name, camelcase, .. }] = tree_sitter_config.grammars.as_slice() else {
      panic!("only single-grammar files supported");
    };

    let mut build = cc::Build::new();
    let src_path = grammar_path.join("src");
    let parser_path = src_path.join("parser.c");
    let scanner_path = src_path.join("scanner.c");
    build
      .include(src_path)
      .opt_level(2) // To ignore FORTIFY_SOURCE warnings
      .flag("-Wno-unused-parameter")
      .flag("-Wno-unused-value")
      .file(parser_path);
    if scanner_path.exists() {
      build.file(scanner_path);
    }
    build.compile(&format!("tree-sitter-{name}"));

    languages.push(camelcase.clone());
    mods.push(format!(
      r#"
pub mod {name} {{
  unsafe extern "C" {{ fn tree_sitter_{name}() -> tree_sitter::Language; }}

  pub fn language() -> tree_sitter::Language {{ unsafe {{ tree_sitter_{name}() }} }}
}}
      "#,
    ));
  }

  let mods = mods.join("\n");
  let languages = languages.join(",");
  let grammars_mod = format!(
    "pub mod grammars {{
    #[derive(Clone, Debug, clap::ValueEnum)]
    pub enum Language {{
      {languages}
    }}

    {mods}
  }}"
  );

  std::fs::write(out_dir.join("grammars.rs"), grammars_mod).unwrap();
}

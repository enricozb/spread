use std::path::PathBuf;

use trix_build::{TrixConfig, Macros};

fn main() {
  println!("cargo:rerun-if-env-changed=TRIX_CONFIG_JSON");

  // Silence C compiler warnings that originate in tree-sitter grammar sources
  // (e.g. -Wsign-compare and -Wimplicit-fallthrough in tree-sitter-html).
  // trix_build compiles grammars via the `cc` crate, which appends $CFLAGS to
  // every compiler invocation, so extending it here is the right hook.
  let cflags = std::env::var("CFLAGS").unwrap_or_default();
  unsafe {
    std::env::set_var(
      "CFLAGS",
      format!("{cflags} -Wno-sign-compare -Wno-implicit-fallthrough"),
    );
  }
  let config = TrixConfig::from_env("TRIX_CONFIG_JSON").unwrap();
  let macros = Macros::from_config(&config).unwrap();
  let out_dir = PathBuf::from(std::env::var("OUT_DIR").unwrap());
  std::fs::write(out_dir.join("grammars.rs"), macros.to_string()).unwrap();
}

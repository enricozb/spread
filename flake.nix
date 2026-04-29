{
  description = "spread";
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    crane.url = "github:ipetkov/crane";
    treefmt-nix = {
      url = "github:numtide/treefmt-nix";
      inputs.nixpkgs.follows = "nixpkgs";
    };

    trix.url = "github:enricozb/trix";
    tree-sitter-kak = {
      url = "github:saifulapm/tree-sitter-kakscript";
      flake = false;
    };
    tree-sitter-markdown = {
      url = "github:tree-sitter-grammars/tree-sitter-markdown";
      flake = false;
    };
    tree-sitter-nix = {
      url = "github:nix-community/tree-sitter-nix";
      flake = false;
    };
    tree-sitter-nushell = {
      url = "github:nushell/tree-sitter-nu";
      flake = false;
    };
    tree-sitter-python = {
      url = "github:tree-sitter/tree-sitter-python";
      flake = false;
    };
    tree-sitter-rust = {
      url = "github:tree-sitter/tree-sitter-rust";
      flake = false;
    };
    tree-sitter-vine = {
      url = "github:VineLang/vine";
      flake = false;
    };
    tree-sitter-css = {
      url = "github:tree-sitter/tree-sitter-css";
      flake = false;
    };
    tree-sitter-html = {
      url = "github:tree-sitter/tree-sitter-html";
      flake = false;
    };
    tree-sitter-javascript = {
      url = "github:tree-sitter/tree-sitter-javascript";
      flake = false;
    };
    tree-sitter-typescript = {
      url = "github:tree-sitter/tree-sitter-typescript";
      flake = false;
    };
  };

  outputs =
    {
      nixpkgs,
      flake-utils,
      rust-overlay,
      crane,
      treefmt-nix,

      trix,
      tree-sitter-css,
      tree-sitter-html,
      tree-sitter-kak,
      tree-sitter-markdown,
      tree-sitter-nix,
      tree-sitter-javascript,
      tree-sitter-typescript,
      tree-sitter-nushell,
      tree-sitter-python,
      tree-sitter-rust,
      tree-sitter-vine,
      ...
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        pkgs = import nixpkgs {
          inherit system;
          overlays = [ rust-overlay.overlays.default ];
        };

        rustToolchain = pkgs.rust-bin.fromRustupToolchainFile ./rust-toolchain.toml;
        craneLib = (crane.mkLib pkgs).overrideToolchain (_: rustToolchain);
        treefmt = treefmt-nix.lib.evalModule pkgs {
          projectRootFile = "flake.nix";
          programs.nixfmt.enable = true;
          programs.rustfmt.enable = true;
        };
        grammars = {
          kak.src = tree-sitter-kak;
          markdown.src = tree-sitter-markdown;
          nix.src = tree-sitter-nix;
          nushell.src = tree-sitter-nushell;
          python.src = tree-sitter-python;
          rust.src = tree-sitter-rust;
          ivy.src = "${tree-sitter-vine}/lsp/tree-sitter-ivy";
          vine.src = "${tree-sitter-vine}/lsp/tree-sitter-vine";
          css.src = tree-sitter-css;
          html.src = tree-sitter-html;
          javascript.src = tree-sitter-javascript;
          typescript = {
            src = tree-sitter-typescript;
            filter = [
              "typescript"
              "tsx"
            ];
          };
        };
        trixLib = trix.mkLib pkgs grammars;
        trixConfig = builtins.toJSON trixLib.config;
      in
      {
        packages.default = craneLib.buildPackage {
          src = craneLib.cleanCargoSource ./.;
          env.TRIX_CONFIG_JSON = trixConfig;
        };

        devShells.default = craneLib.devShell {
          TRIX_CONFIG_JSON = trixConfig;
          packages = [ pkgs.tree-sitter ];
        };

        formatter = treefmt.config.build.wrapper;
      }
    );
}

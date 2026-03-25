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
  };

  outputs =
    {
      nixpkgs,
      flake-utils,
      rust-overlay,
      crane,
      treefmt-nix,

      tree-sitter-kak,
      tree-sitter-markdown,
      tree-sitter-nix,
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
        grammars = builtins.concatStringsSep ":" (pkgs.lib.mapAttrsToList tree-sitter-build grammar-srcs);
        grammar-srcs = {
          kak = tree-sitter-kak;
          markdown = tree-sitter-markdown;
          nix = tree-sitter-nix;
          nushell = tree-sitter-nushell;
          python = tree-sitter-python;
          rust = tree-sitter-rust;
          ivy = "${tree-sitter-vine}/lsp/tree-sitter-ivy";
          vine = "${tree-sitter-vine}/lsp/tree-sitter-vine";
        };
        tree-sitter-build =
          name: src:
          pkgs.stdenv.mkDerivation {
            name = "tree-sitter-${name}";
            inherit src;
            nativeBuildInputs = [
              pkgs.jq
              pkgs.tree-sitter
              pkgs.nodejs_24
            ];
            configurePhase = ''
              echo 'skipping configure'
            '';
            buildPhase = ''
              for grammar_path in $(jq '.grammars[].path // "."' tree-sitter.json -r); do
                tree-sitter generate "$grammar_path/grammar.js"
              done
            '';
            installPhase = ''
              mkdir $out
              cp tree-sitter.json $out

              for grammar_path in $(jq '.grammars[].path // "."' tree-sitter.json -r); do
                echo checking $grammar_path

                ls "$grammar_path"
                mkdir -p "$out/$grammar_path"
                cp -r "$grammar_path/src" "$out/$grammar_path"
              done
            '';
          };
      in
      {
        packages.default = craneLib.buildPackage {
          src = craneLib.cleanCargoSource ./.;
          env.GRAMMARS = grammars;
        };

        devShells.default = craneLib.devShell {
          GRAMMARS = grammars;
          packages = [ pkgs.tree-sitter ];
        };

        formatter = treefmt.config.build.wrapper;
      }
    );
}

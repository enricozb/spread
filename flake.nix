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
          inherit
            tree-sitter-nushell
            tree-sitter-rust
            tree-sitter-nix
            tree-sitter-python
            ;
          tree-sitter-vine = "${tree-sitter-vine}/lsp/tree-sitter-vine";
        };
        tree-sitter-build =
          name: src:
          pkgs.stdenv.mkDerivation {
            inherit name src;
            nativeBuildInputs = [
              pkgs.tree-sitter
              pkgs.nodejs_24
            ];
            configurePhase = ''
              echo 'skipping configure'
            '';
            buildPhase = ''
              tree-sitter generate
            '';
            installPhase = ''
              mkdir $out
              cp -r tree-sitter.json src/ $out
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

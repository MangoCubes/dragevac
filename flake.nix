{
  description = "Rust development environment";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay.url = "github:oxalica/rust-overlay";
  };

  outputs =
    {
      self,
      nixpkgs,
      flake-utils,
      rust-overlay,
    }:
    {
      homeManager = {
        default = ./nix/homeManager.nix;
      };
    }
    // flake-utils.lib.eachDefaultSystem (
      system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs {
          inherit system overlays;
        };
        rustToolchain = pkgs.rust-bin.stable.latest.default.override {
          extensions = [
            "rust-src"
            "rust-analyzer"
          ];
        };
        # Things needed at build time only, something that doesn't have to exist on the system when running the compiled program
        # `pkg-config` is a helper tool used when compiling applications and libraries, so it is only needed when the program is being compiled
        nativeBuildInputs = [ pkgs.pkg-config ];
        # Things needed at runtime, something that must be installed on the computer in order to run the program
        buildInputs =
          (with pkgs; [
            gtk4
            gtk4-layer-shell
            glib
          ])
          ++ [ rustToolchain ];
      in
      {
        packages.default = pkgs.rustPlatform.buildRustPackage {
          inherit buildInputs nativeBuildInputs;
          src = ./.;
          name = "dragevac";
          cargoHash = "sha256-SEjkdAXjMPoIUqno0aswVQK3sv7wrIxDItDwhFIQk6w=";
        };

        devShells.default = pkgs.mkShell {
          inherit nativeBuildInputs buildInputs;
          packages = (
            with pkgs;
            [

              # This package is needed to ensure subsequent shells looks pretty
              bashInteractive
            ]
          );
          shellHook =
            let
              initFile = pkgs.writeText ".bashrc" ''
                echo "Activating Rust develop environment..."
                set -a
                  hw() { echo "Hello world!"; }
                set +a
              '';
            in
            ''
              bash --init-file ${initFile}; exit
            '';
        };
      }
    );
}

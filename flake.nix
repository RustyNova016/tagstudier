{
  description = "Rust devShell";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
    flake-utils.url = "github:numtide/flake-utils";
    cargo-berger.url = "github:RustyNova016/cargo-berger/f622a8db0a1093547a95a2d041c7649bfef9b367";
  };

  outputs =
    {
      nixpkgs,
      rust-overlay,
      flake-utils,
      cargo-berger,
      ...
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        overlays = [ (import rust-overlay) ];

        pkgs = import nixpkgs {
          inherit system overlays;
        };
      in
      rec {
        # Executed by `nix build`
        packages.default = pkgs.rustPlatform.buildRustPackage {
          pname = "tagstudier";
          version = "0.1.0";
          src = pkgs.lib.cleanSource ./.;
          cargoLock.lockFile = ./Cargo.lock;

          buildInputs = [
            pkgs.openssl
          ];

          nativeBuildInputs = [
            pkgs.pkg-config
          ];

          # For other makeRustPlatform features see:
          # https://github.com/NixOS/nixpkgs/blob/master/doc/languages-frameworks/rust.section.md#cargo-features-cargo-features
        };

        devShells.default =
          with pkgs;
          mkShell {
            buildInputs = [
              openssl
              pkg-config
              pkgs.gh

              # CI / Linting tools
              cargo-mutants
              cargo-hack
              cargo-msrv
              cargo-audit
              cargo-machete
              cargo-berger.packages."${pkgs.stdenv.hostPlatform.system}".default
              

              (rust-bin.stable.latest.default.override {
                extensions = [
                  "cargo"
                  "clippy"
                  "rust-src"
                  "rust-analyzer"
                ];
              })
            ];
          };
      }
    );
}
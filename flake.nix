{
  description = "Project: ABBA";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-23.11";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs = {
        nixpkgs.follows = "nixpkgs";
        flake-utils.follows = "flake-utils";
      };
    };
  };

  outputs = { self, nixpkgs, flake-utils, rust-overlay }: 
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs {
          inherit system overlays;
        };
        rustToolchain = (pkgs.pkgsBuildHost.rust-bin.stable.latest.default).override {
          extensions = [ "rust-src" "clippy" "rustfmt" "rust-analyzer"];
        };

        # During build time
        nativeBuildInputs = with pkgs; [ 
          rustPlatform.bindgenHook 
          rustToolchain
        ];
        # During runtime
        buildInputs = with pkgs; [
        ];
      in
      with pkgs;
      {
        devShells.default = mkShell {
          inherit buildInputs nativeBuildInputs;

          RUST_SRC_PATH="${rustToolchain}/lib/rustlib/src/rust/src";
        };
      }
    );
}

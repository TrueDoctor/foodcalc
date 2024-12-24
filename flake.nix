{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
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
    let
      overlays = [ (import rust-overlay) ];
      forAllSystems = nixpkgs.lib.genAttrs nixpkgs.lib.systems.flakeExposed;
    in {
      devShell = forAllSystems ( system:
        let 
          pkgs = import nixpkgs { inherit system overlays; };
          toolchain = pkgs.rust-bin.stable.latest.default.override {
            extensions = ["rust-src" "clippy" "rust-analyzer"];
          };
          buildInputs = with pkgs; [
              openssl
              icu
          ];        
        in pkgs.mkShell {
          packages = [
            pkgs.bacon
            pkgs.sqlx-cli
            pkgs.cargo-udeps
          ];
          nativeBuildInputs = with pkgs; [
            toolchain
            nodejs
            cargo
            pkg-config
            cargo-nextest
         ];
         inherit buildInputs;
          
          LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath buildInputs;
        }
      );
    };
}


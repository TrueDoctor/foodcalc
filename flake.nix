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

  outputs = {self, nixpkgs, flake-utils, rust-overlay }:
    flake-utils.lib.eachDefaultSystem ( system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs {
          inherit system overlays;
        };
        toolchain = pkgs.rust-bin.stable.latest.default.override {
          extensions = ["rust-src" "clippy" "rust-analyzer"];
        };
        buildInputs = with pkgs; [
            openssl
            expat
            fontconfig
            freetype
            icu
            graphite2
            stdenv.cc.cc.lib
            libpng
            zlib

            libGL
            vulkan-loader
            wayland
            wayland-protocols
            libxkbcommon
        ];        
        in with pkgs; {
        devShells.default = mkShell {
          packages = [
            pkgs.bacon
          ];
          nativeBuildInputs = with pkgs; [
            toolchain
            nodejs
            cargo
            pkg-config
         ];
         inherit buildInputs;
          
          LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath buildInputs;
        };
      }
    );
}

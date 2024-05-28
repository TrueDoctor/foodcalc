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
      #packages = forAllSystems (system:
      #  let pkgs = import nixpkgs { inherit system overlays; };
      #  in rec {
      #    default = pkgs.stdenv.mkDerivation {
      #      pname = "foodcalc";
      #      version = "1.0.0";
      #      src = ./cli-client; # Path to the cli-client directory
      #      cargoSha256 = "..."; # SHA256 hash of your source code
      #      cargoBuildFlags = [
      #        "--release"
      #      ];
      #      nativeBuildInputs = [
      #        pkgs.rustc
      #        pkgs.cargo
      #      ];
      #      buildInputs = [
      #        pkgs.openssl
      #        pkgs.expat
      #        pkgs.fontconfig
      #        pkgs.freetype
      #        pkgs.libGL
      #        pkgs.vulkan-loader
      #        pkgs.wayland
      #        pkgs.wayland-protocols
      #        pkgs.libxkbcommon
      #      ];
      #      outputs = [ "out" ];
      #      installPhase = ''
      #        mkdir -p $out/bin
      #        cp target/release/foodctl $out/bin/
      #      '';
      #    };
      #  }
      #);
      devShell = forAllSystems ( system:
        let 
          pkgs = import nixpkgs { inherit system overlays; };
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
        in pkgs.mkShell {
          packages = [
            pkgs.bacon
            pkgs.sqlx-cli
          ];
          nativeBuildInputs = with pkgs; [
            toolchain
            nodejs
            cargo
            pkg-config
         ];
         inherit buildInputs;
          
          LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath buildInputs;
        }
      );
    };
}


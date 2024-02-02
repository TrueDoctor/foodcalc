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
          targets = [ "wasm32-unknown-unknown" ];
        };
        rustc-wasm = pkgs.rust-bin.stable.latest.default.override {
          targets = [ "wasm32-unknown-unknown" ];
          # wasm-pack needs this
          extensions = [ "rust-src" ];
        };
      in with pkgs; {
        devShells.default = mkShell {
          packages = [
            pkgs.bacon
          ];
          buildInputs = with pkgs; [
            rustc-wasm
            nodejs
            cargo
            cargo-watch
            wasm-pack
            clang
            tailwindcss

            openssl
            openssl.dev
            glib
            gtk3
            libsoup
            webkitgtk
            freetype
            freetype.dev

            pkg-config

            # Use Mold as a Linke
            mold

            # Vulkan
            #pkgs.glslang
            shaderc
            vulkan-headers
            vulkan-loader
            #pkgs.vulkan-validation-layers
          ];

          # Hacky way to run cago through Mold
          shellHook = ''
          alias cargo='mold --run cargo'
          '';
          LD_LIBRARY_PATH = lib.makeLibraryPath [ 
            pkgs.openssl
            pkgs.freetype
            pkgs.fontconfig
            pkgs.expat
            pkgs.libGL
          ];

          #VULKAN_SDK = "${pkgs.vulkan-validation-layers}/share/vulkan/explicit_layer.d";
        };
      }
    );
}

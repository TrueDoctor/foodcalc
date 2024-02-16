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
        buildInputs = with pkgs; [
            openssl
            expat
            fontconfig
            freetype

            libGL
            vulkan-loader
            wayland
            wayland-protocols
            libxkbcommon
        ];        
        in with pkgs; {
        devShells.default = mkShell {
          packages = [
            pkgs.sqlx-cli
            pkgs.bacon
            pkgs.postgresql
          ];
          nativeBuildInputs = with pkgs; [
            rustc
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

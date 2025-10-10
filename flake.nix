{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    fenix.url = "github:nix-community/fenix";
    fenix.inputs.nixpkgs.follows = "nixpkgs";
  };
  outputs =
    {
      self,
      nixpkgs,
      flake-utils,
      fenix,
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        pkgs = import nixpkgs {
          system = system;
        };
        packages = with pkgs; [
          cargo-info
          cargo-udeps
          just
          (
            with fenix.packages.${system};
            combine [
              complete.rustc
              complete.rust-src
              complete.cargo
              complete.clippy
              complete.rustfmt
              complete.rust-analyzer
            ]
          )
        ];
        libraries = with pkgs; [
          pkg-config
          pango
          cairo
          glib
          atk
          gdk-pixbuf
          gtk3
        ];
      in
      {
        devShell = pkgs.mkShell {
          buildInputs = packages ++ libraries;
          nativeBuildInputs = [ pkgs.pkg-config ];
          PKG_CONFIG_PATH = "${pkgs.lib.makeLibraryPath libraries}/../lib/pkgconfig";
          LD_LIBRARY_PATH = "${pkgs.lib.makeLibraryPath libraries}";
        };
      }
    );
}

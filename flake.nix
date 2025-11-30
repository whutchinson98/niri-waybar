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
        
        rustToolchain = with fenix.packages.${system}; combine [
          complete.rustc
          complete.rust-src
          complete.cargo
          complete.clippy
          complete.rustfmt
          complete.rust-analyzer
        ];
        
        packages = with pkgs; [
          cargo-info
          cargo-udeps
          just
          rustToolchain
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
        
        niri-waybar = pkgs.rustPlatform.buildRustPackage {
          pname = "niri-waybar";
          version = "0.4.0";
          
          src = ./.;
          
          cargoLock = {
            lockFile = ./Cargo.lock;
          };
          
          nativeBuildInputs = with pkgs; [
            pkg-config
            rustToolchain
          ];
          
          buildInputs = libraries;
          
          # Build as cdylib
          buildPhase = ''
            cargo build --release --lib
          '';
          
          installPhase = ''
            mkdir -p $out/lib
            cp target/release/libniri_waybar.so $out/lib/
          '';
          
          PKG_CONFIG_PATH = "${pkgs.lib.makeLibraryPath libraries}/../lib/pkgconfig";
        };
        
      in
      {
        packages = {
          default = niri-waybar;
          niri-waybar = niri-waybar;
        };
        
        devShell = pkgs.mkShell {
          buildInputs = packages ++ libraries;
          nativeBuildInputs = [ pkgs.pkg-config ];
          PKG_CONFIG_PATH = "${pkgs.lib.makeLibraryPath libraries}/../lib/pkgconfig";
          LD_LIBRARY_PATH = "${pkgs.lib.makeLibraryPath libraries}";
        };
      }
    );
}

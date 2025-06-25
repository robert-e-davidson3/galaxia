{
  description = "Galaxia - A Rust game built with Bevy";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, rust-overlay, flake-utils }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs {
          inherit system overlays;
        };
        
        rustVersion = pkgs.rust-bin.stable.latest.default.override {
          extensions = [ "rust-src" "rustfmt" "clippy" ];
        };

        nativeBuildInputs = with pkgs; [
          rustVersion
          pkg-config
        ];

        buildInputs = with pkgs; [
          # Bevy dependencies
          alsa-lib
          udev
          vulkan-loader
          libxkbcommon
          wayland
          
          # X11 dependencies
          xorg.libX11
          xorg.libXcursor
          xorg.libXi
          xorg.libXrandr
          
          # Additional graphics dependencies
          libGL
        ] ++ lib.optionals stdenv.isDarwin [
          darwin.apple_sdk.frameworks.Cocoa
          darwin.apple_sdk.frameworks.CoreAudio
          darwin.apple_sdk.frameworks.AudioUnit
        ];

        rustPlatform = pkgs.makeRustPlatform {
          cargo = rustVersion;
          rustc = rustVersion;
        };

      in
      {
        packages.default = rustPlatform.buildRustPackage {
          pname = "galaxia";
          version = "0.1.0";

          src = ./.;

          cargoLock = {
            lockFile = ./Cargo.lock;
          };

          inherit nativeBuildInputs buildInputs;

          # Set environment variables for graphics libraries
          LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath buildInputs;
          
          meta = with pkgs.lib; {
            description = "A Rust game built with Bevy";
            license = licenses.mit;
            maintainers = [ ];
          };
        };

        devShells.default = pkgs.mkShell {
          inherit buildInputs nativeBuildInputs;
          
          shellHook = ''
            export LD_LIBRARY_PATH=${pkgs.lib.makeLibraryPath buildInputs}:$LD_LIBRARY_PATH
            export PKG_CONFIG_PATH=${pkgs.lib.makeSearchPath "lib/pkgconfig" buildInputs}:$PKG_CONFIG_PATH
          '';
        };
      });
}
{
  description = "Galaxia - A Rust game built with Bevy";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-25.11";
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
          zsh
        ];

        buildInputs = with pkgs; [
          # Bevy dependencies
          alsa-lib
          udev
          vulkan-loader
          libxkbcommon

          # Wayland (winit links wayland-client by default since Bevy 0.17)
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

      in
      {
        devShells.default = pkgs.mkShell {
          inherit buildInputs nativeBuildInputs;
          
          shellHook = ''
            export LD_LIBRARY_PATH=${pkgs.lib.makeLibraryPath buildInputs}:$LD_LIBRARY_PATH
            export PKG_CONFIG_PATH=${pkgs.lib.makeSearchPath "lib/pkgconfig" buildInputs}:$PKG_CONFIG_PATH
            export NIX_SHELL_ENV=dev
            # Bevy 0.18's GPU-preprocessing compute shaders use relaxed-order
            # atomics that the Vulkan validation layer (stricter since the
            # nixos-25.11 pin) reports as spec violations. RADV runs them fine,
            # so disable the layer outright to keep dev runs free of those
            # non-fatal ERROR-level reports. (We don't write raw Vulkan — Bevy
            # does — so we lose nothing by dropping validation here.)
            export VK_LOADER_LAYERS_DISABLE=VK_LAYER_KHRONOS_validation
            # Only drop into an interactive zsh when the shell is entered
            # interactively. For `nix develop --command ...` (non-interactive)
            # skip it so the given command actually runs.
            if [[ $- == *i* ]]; then
              exec zsh
            fi
          '';
        };
      });
}

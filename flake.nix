{
  description = "Keyboard-based mouse control using uinput";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs =
    { self
    , nixpkgs
    , rust-overlay
    , flake-utils
    , ...
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs {
          inherit system overlays;
        };

        rustVersion = pkgs.rust-bin.stable.latest.default;

        rustPlatform = pkgs.makeRustPlatform {
          cargo = rustVersion;
          rustc = rustVersion;
        };

        nativeBuildInputs = with pkgs; [
          rustVersion
          pkg-config
        ];

        buildInputs = with pkgs; [
          # Linux input libraries
          linuxHeaders
          systemd.dev

          # Development tools
          rust-analyzer
          rustfmt
          clippy
          evtest
        ];

      in
      {
        devShells.default = pkgs.mkShell {
          inherit nativeBuildInputs buildInputs;

          shellHook = ''
            echo "Rust development environment loaded"
            echo "This environment includes all dependencies for keyboard-based mouse control"
          '';
        };
      }
    );
}

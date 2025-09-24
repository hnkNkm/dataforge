{
  description = "DataForge - Tauri DB Client Application";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = { self, nixpkgs, flake-utils, rust-overlay }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs {
          inherit system overlays;
        };

        rustToolchain = pkgs.rust-bin.stable.latest.default.override {
          extensions = [ "rust-src" "rust-analyzer" ];
        };
      in
      {
        devShells.default = pkgs.mkShell {
          buildInputs = with pkgs; [
            # Rust
            rustToolchain
            cargo-tauri
            pkg-config

            # Node.js
            nodejs_22
            pnpm

            # Tauri dependencies for Linux
          ] ++ pkgs.lib.optionals pkgs.stdenv.isLinux [
            webkitgtk_4_1
            gtk3
            cairo
            gdk-pixbuf
            glib
            dbus
            openssl_3
            librsvg
            libsoup_3
          ] ++ pkgs.lib.optionals pkgs.stdenv.isDarwin [
            # macOS specific dependencies
            libiconv
          ];

          shellHook = ''
            echo "DataForge Development Environment"
            echo "Node.js: $(node --version)"
            echo "pnpm: $(pnpm --version)"
            echo "Rust: $(rustc --version)"
            echo ""
            echo "Available commands:"
            echo "  pnpm install         - Install dependencies"
            echo "  pnpm tauri dev       - Start development server"
            echo "  pnpm tauri build     - Build for production"
          '';

          # Environment variables for Tauri
          WEBKIT_DISABLE_DMABUF_RENDERER = "1";
        };
      });
}
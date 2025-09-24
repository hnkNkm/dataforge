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

            # Database
            postgresql_16
            mysql80
            sqlite

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
            echo "PostgreSQL: $(postgres --version)"
            echo "MySQL: $(mysql --version)"
            echo ""
            echo "Available commands:"
            echo "  pnpm install         - Install dependencies"
            echo "  pnpm tauri dev       - Start development server"
            echo "  pnpm tauri build     - Build for production"
            echo ""
            echo "Database commands:"
            echo "  PostgreSQL:"
            echo "    pg_ctl init -D ./database/postgres     - Initialize PostgreSQL database"
            echo "    pg_ctl start -D ./database/postgres    - Start PostgreSQL server"
            echo "    pg_ctl stop -D ./database/postgres     - Stop PostgreSQL server"
            echo "    psql -d postgres                        - Connect to PostgreSQL"
            echo "  MySQL:"
            echo "    mysqld --initialize-insecure --datadir=./database/mysql  - Initialize MySQL"
            echo "    mysqld --datadir=./database/mysql &                      - Start MySQL server"
            echo "    mysql -u root                                             - Connect to MySQL"
            echo "  SQLite:"
            echo "    sqlite3 ./database/sqlite/dataforge.db - Create/Connect to SQLite database"
          '';

          # Environment variables for Tauri
          WEBKIT_DISABLE_DMABUF_RENDERER = "1";
        };
      });
}
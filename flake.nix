{
  description = "Autobank - Rule-based banking automation";

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
        pkgs = import nixpkgs {
          inherit system;
          overlays = [ rust-overlay.overlays.default ];
        };
        
        rustToolchain = pkgs.rust-bin.stable.latest.default.override {
          extensions = [ "rust-src" "rust-analyzer" ];
        };
      in {
        devShells.default = pkgs.mkShell {
          buildInputs = with pkgs; [
            # Rust toolchain
            rustToolchain
            cargo-watch
            cargo-nextest
            
            # Database
            sqlite
            sqlx-cli
            
            # Node.js for frontend
            nodejs_22
            pnpm
            
            # Development tools
            just
            openssl
            pkg-config
            
            # Optional: for testing
            httpie
            jq
          ];
          
          env = {
            DATABASE_URL = "sqlite:./autobank.db";
            RUST_LOG = "info,autobank=debug";
          };
          
          shellHook = ''
            echo "Autobank development environment"
            echo "Commands:"
            echo "  just dev      - Run backend in dev mode"
            echo "  just web      - Run frontend in dev mode"
            echo "  just test     - Run all tests"
          '';
        };
        
        packages = {
          default = self.packages.${system}.autobank-server;
          
          autobank-server = pkgs.rustPlatform.buildRustPackage {
            pname = "autobank-server";
            version = "0.1.0";
            src = ./.;
            cargoLock.lockFile = ./Cargo.lock;
            
            nativeBuildInputs = [ pkgs.pkg-config ];
            buildInputs = [ pkgs.openssl pkgs.sqlite ];
          };
        };
      }
    );
}

{
  description = "CIM Network - Network Topology Builder with MCP Server";

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

        # Python package for MCP server
        cim-network-mcp = pkgs.python3.pkgs.buildPythonPackage rec {
          pname = "cim-network-mcp";
          version = "1.0.0";
          src = ./.;
          
          propagatedBuildInputs = with pkgs.python3.pkgs; [
            # No external MCP dependencies needed for our simple server
          ];

          # Don't run tests during build (they require Rust compilation)
          doCheck = false;

          meta = with pkgs.lib; {
            description = "MCP server for interactive network topology building";
            license = licenses.mit;
          };
        };

        # Rust crate
        cim-network-rust = pkgs.rustPlatform.buildRustPackage rec {
          pname = "cim-network";
          version = "0.1.0";
          src = ./.;

          cargoLock = {
            lockFile = ./Cargo.lock;
            allowBuiltinFetchGit = true;
          };

          nativeBuildInputs = with pkgs; [
            pkg-config
          ];

          buildInputs = with pkgs; [
            openssl
          ] ++ pkgs.lib.optionals pkgs.stdenv.isDarwin [
            pkgs.darwin.apple_sdk.frameworks.Security
            pkgs.darwin.apple_sdk.frameworks.SystemConfiguration
          ];

          # Build only the subagent demo for now
          cargoBuildFlags = [ "--example" "subagent_demo" ];

          meta = with pkgs.lib; {
            description = "Network infrastructure management for CIM";
            license = licenses.mit;
          };
        };

      in
      {
        packages = {
          default = cim-network-mcp;
          mcp-server = cim-network-mcp;
          rust-agent = cim-network-rust;
        };

        devShells.default = pkgs.mkShell {
          buildInputs = with pkgs; [
            # Rust toolchain
            rustToolchain
            pkg-config
            openssl

            # Python for MCP server
            python3
            python3.pkgs.pip

            # Development tools
            git
            jq
            
            # MCP testing tools
            nodejs # for claude-code if needed
          ];

          shellHook = ''
            echo "🌐 CIM Network Development Environment"
            echo "====================================="
            echo ""
            echo "Available commands:"
            echo "• cargo run --example subagent_demo    - Run the Rust sub-agent demo"
            echo "• python3 -m cim_network_mcp          - Start the MCP server"
            echo "• cargo build --release                - Build optimized Rust components"
            echo ""
            echo "MCP Server Configuration for Claude Code:"
            echo "Add this to your Claude Code MCP settings:"
            echo ""
            cat .claude/mcp_settings.json
            echo ""
            echo "🦀 Rust toolchain: $(rustc --version)"
            echo "🐍 Python: $(python3 --version)"
          '';

          RUST_SRC_PATH = "${rustToolchain}/lib/rustlib/src/rust/library";
        };

        # Claude Code MCP configuration
        apps = {
          mcp-server = {
            type = "app";
            program = "${pkgs.python3.withPackages (ps: [ cim-network-mcp ])}/bin/python3";
            args = [ "-m" "cim_network_mcp" ];
          };
        };

        # NixOS module for system-wide installation
        nixosModules.default = { config, lib, pkgs, ... }: {
          options.services.cim-network-mcp = {
            enable = lib.mkEnableOption "CIM Network MCP Server";
            
            package = lib.mkOption {
              type = lib.types.package;
              default = cim-network-mcp;
              description = "The CIM Network MCP package to use";
            };

            workingDirectory = lib.mkOption {
              type = lib.types.path;
              default = "/var/lib/cim-network";
              description = "Working directory for the MCP server";
            };
          };

          config = lib.mkIf config.services.cim-network-mcp.enable {
            systemd.services.cim-network-mcp = {
              description = "CIM Network MCP Server";
              wantedBy = [ "multi-user.target" ];
              
              serviceConfig = {
                Type = "simple";
                ExecStart = "${config.services.cim-network-mcp.package}/bin/python3 -m cim_network_mcp";
                WorkingDirectory = config.services.cim-network-mcp.workingDirectory;
                Restart = "always";
                RestartSec = 5;
                
                # Security hardening
                NoNewPrivileges = true;
                ProtectSystem = "strict";
                ProtectHome = true;
                ReadWritePaths = [ config.services.cim-network-mcp.workingDirectory ];
              };
            };

            # Ensure working directory exists
            systemd.tmpfiles.rules = [
              "d ${config.services.cim-network-mcp.workingDirectory} 0755 nobody nogroup -"
            ];
          };
        };
      });
}
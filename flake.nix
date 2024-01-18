{
  description = "teywi flake";

  inputs = {
    flake-parts.url = "github:hercules-ci/flake-parts";
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs = {
        nixpkgs.follows = "nixpkgs";
      };
    };
  };

  outputs = { self, nixpkgs, flake-parts, rust-overlay }@inputs:
    flake-parts.lib.mkFlake { inherit inputs; } {
      imports = [
        inputs.flake-parts.flakeModules.easyOverlay
      ];

      systems = [ "x86_64-linux" "aarch64-linux" "aarch64-darwin" "x86_64-darwin" ];

      perSystem = { config, self', inputs', pkgs, system, ... }:
        {
          packages = {
            rust-stable = inputs'.rust-overlay.packages.rust.override {
              extensions = [ "rust-src" "rust-analyzer" "clippy" ];
            };
          };

          devShells.default = with pkgs; mkShell {
            buildInputs = [
              self'.packages.rust-stable
              vscode-extensions.vadimcn.vscode-lldb

              gh
              nil
              nixpkgs-fmt
              nodejs_20
              nodePackages.markdownlint-cli
              nodePackages.prettier
            ];
            shellHook = ''
              export PATH=~/.cargo/bin:$PATH
              export PATH=${vscode-extensions.vadimcn.vscode-lldb}/share/vscode/extensions/vadimcn.vscode-lldb/adapter:$PATH
            '';

            RUST_BACKTRACE = "full";
          };
        };
    };
}

# https://github.com/cpu/woodwidelog/blob/bb549af2b33c5c50ae6e7361da4af3b1993caa1d/content/articles/rust-flake/index.md?plain=1#L50
{
  description = "yate flake";

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
        let
          overlays = [ (import rust-overlay) ];
          pkgs = import nixpkgs {
            inherit system overlays;
          };

          toml = builtins.fromTOML (builtins.readFile ./yate/Cargo.toml);

          package = (pkgs.makeRustPlatform {
            cargo = pkgs.rust-bin.stable.latest.minimal;
            rustc = pkgs.rust-bin.stable.latest.minimal;
          }).buildRustPackage {
            inherit (toml.package) name version;
            src = ./.;
            cargoLock.lockFile = ./Cargo.lock;
          };

          rust-stable = inputs'.rust-overlay.packages.rust.override {
            extensions = [ "rust-src" "rust-analyzer" "clippy" ];
          };

          shell = pkgs.mkShell {
            nativeBuildInputs = [
              rust-stable
              pkgs.vscode-extensions.vadimcn.vscode-lldb
              pkgs.gh
              pkgs.nil
              pkgs.nixpkgs-fmt
              pkgs.nodejs_20
              pkgs.nodePackages.markdownlint-cli
              pkgs.nodePackages.prettier
            ];
            shellHook = ''
              export PATH=~/.cargo/bin:$PATH
              export PATH=${pkgs.vscode-extensions.vadimcn.vscode-lldb}/share/vscode/extensions/vadimcn.vscode-lldb/adapter:$PATH
            '';

            RUST_BACKTRACE = "full";
          };
        in
        {
          packages = {
            default = self'.packages.yate;
            yate = package;
          };

          devShells.default = shell;
        };
    };
}

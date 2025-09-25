# https://github.com/cpu/woodwidelog/blob/bb549af2b33c5c50ae6e7361da4af3b1993caa1d/content/articles/rust-flake/index.md?plain=1#L50
{
  description = "yeet the great flake";

  inputs = {
    flake-parts.url = "github:hercules-ci/flake-parts";
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
    nixgl = {
      url = "github:nix-community/nixGL";
      inputs = {
        nixpkgs.follows = "nixpkgs";
      };
    };
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs = {
        nixpkgs.follows = "nixpkgs";
      };
    };
  };

  outputs = { self, nixpkgs, nixgl, flake-parts, rust-overlay, ... }@inputs:
    flake-parts.lib.mkFlake { inherit inputs; } {
      imports = [
        inputs.flake-parts.flakeModules.easyOverlay
      ];

      systems = [ "x86_64-linux" "aarch64-linux" "aarch64-darwin" "x86_64-darwin" ];

      perSystem = { config, self', inputs', pkgs, system, lib, ... }:
        let
          overlays = [ (import rust-overlay) nixgl.overlay ];
          pkgs = import nixpkgs {
            inherit system overlays;
          };

          toml_version = builtins.fromTOML (builtins.readFile ./Cargo.toml);
          toml_name = builtins.fromTOML (builtins.readFile ./yeet/Cargo.toml);

          package = (pkgs.makeRustPlatform {
            cargo = pkgs.rust-bin.stable.latest.minimal;
            rustc = pkgs.rust-bin.stable.latest.minimal;
          }).buildRustPackage {
            inherit (toml_name.package) name;
            inherit (toml_version.workspace.package) version;

            src = ./.;
            cargoLock.lockFile = ./Cargo.lock;
          };

          rust-stable = inputs'.rust-overlay.packages.rust.override {
            extensions = [ "rust-src" "rust-analyzer" "clippy" ];
          };

          shell = pkgs.mkShell {
            nativeBuildInputs = [
              rust-stable

              pkgs.asciinema
              pkgs.asciinema-agg
              pkgs.gh
              pkgs.nil
              pkgs.nixpkgs-fmt
              pkgs.nodejs_20
              pkgs.nodePackages.markdownlint-cli
              pkgs.nodePackages.prettier

              pkgs.chafa
              pkgs.fd
              pkgs.kitty
              pkgs.wezterm
            ] ++ lib.optionals (!pkgs.stdenv.isDarwin) [
              pkgs.vscode-extensions.vadimcn.vscode-lldb
            ];
            shellHook = ''
              export PATH=~/.cargo/bin:$PATH

              ${ if (!pkgs.stdenv.isDarwin) then
                  "export PATH=${pkgs.vscode-extensions.vadimcn.vscode-lldb}/share/vscode/extensions/vadimcn.vscode-lldb/adapter:$PATH"
                else
                  ""
              }
            '';

            RUST_BACKTRACE = "full";
          };
        in
        {
          overlayAttrs = {
            inherit (config.packages) yeet;
          };

          packages = {
            default = self'.packages.yeet;
            yeet = package;
          };

          devShells.default = shell;
        };
    };
}

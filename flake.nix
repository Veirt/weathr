{
  description = "A terminal weather app with ascii animations.";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-parts.url = "github:hercules-ci/flake-parts";
  };

  outputs =
    inputs@{ flake-parts, ... }:
    flake-parts.lib.mkFlake { inherit inputs; } {
      systems = [
        "x86_64-linux"
        "aarch64-linux"
        "aarch64-darwin"
        "x86_64-darwin"
      ];
      perSystem =
        {
          config,
          self',
          inputs',
          pkgs,
          ...
        }:
        let
          package = pkgs.rustPlatform.buildRustPackage {
            pname = "weathr";
            version = "1.3.0";
            src = ./.;
            cargoLock.lockFile = ./Cargo.lock;

            # skip tests (network integration tests can't be completed inside nix build sandbox environment)
            doCheck = false;
          };
        in
        {
          packages.default = package;

          devShells.default = pkgs.mkShell {
            inputsFrom = [ package ];
            packages = with pkgs; [
              rustc
              cargo
              rust-analyzer
              rustfmt
            ];
          };
        };
    };
}

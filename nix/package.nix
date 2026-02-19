{
  perSystem =
    { pkgs, lib, ... }:
    rec {
      packages.weathr = pkgs.rustPlatform.buildRustPackage (finalAttrs: {
        pname = "weathr";
        version = "1.3.0";
        src = ../.;
        cargoLock.lockFile = ../Cargo.lock;

        nativeBuildInputs = [
          pkgs.installShellFiles
        ];

        postInstall = lib.optionalString (pkgs.stdenv.buildPlatform.canExecute pkgs.stdenv.hostPlatform) ''
          installShellCompletion --cmd weathr \
            --bash <($out/bin/weathr --completions bash) \
            --zsh <($out/bin/weathr --completions zsh) \
            --fish <($out/bin/weathr --completions fish)
        '';

        # skip tests (network integration tests can't be completed inside nix build sandbox environment)
        doCheck = false;
      });

      packages.default = packages.weathr;
    };
}

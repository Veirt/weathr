{
  perSystem =
    { pkgs, ... }:
    rec {
      packages.weathr = pkgs.rustPlatform.buildRustPackage {
        pname = "weathr";
        version = "1.3.0";
        src = ../.;
        cargoLock.lockFile = ../Cargo.lock;

        # skip tests (network integration tests can't be completed inside nix build sandbox environment)
        doCheck = false;
      };

      packages.default = packages.weathr;
    };
}

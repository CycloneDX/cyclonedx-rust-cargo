{
  description = "A framework for building and using custom tools for Sonatype Lift";

  inputs = {
    utils.url = "github:numtide/flake-utils";
    naersk.url = "github:nmattia/naersk";
  };

  outputs = { self, nixpkgs, utils, naersk }:
    utils.lib.eachDefaultSystem (system: let
      pkgs = nixpkgs.legacyPackages."${system}";
      naersk-lib = naersk.lib."${system}";
    in rec {
      # `nix build`
      packages.cyclonedx-rust-cargo = naersk-lib.buildPackage {
        pname = "cyclonedx-rust-cargo";
        root = ./.;
        doCheck = true;
        doDoc = true;
        doDocFail = true;

        buildInputs = with pkgs; [openssl];
        nativeBuildInputs = with pkgs; [pkg-config];
      };
      defaultPackage = packages.cyclonedx-rust-cargo;

      # `nix run`
      apps.cargo-cyclonedx = utils.lib.mkApp {
        drv = packages.cyclonedx-rust-cargo;
        name = "cargo-cyclonedx";
      };
      defaultApp = apps.cargo-cyclonedx;

      # `nix develop`
      devShell = pkgs.mkShell {
        nativeBuildInputs = with pkgs; [
          cargo
          cargo-edit
          cargo-msrv
          cargo-outdated
          clippy
          rustc
          rustfmt
          rust-analyzer

          # Required for compiling OpenSSL
          openssl
          pkg-config

          # GitHub tooling
          gh
        ];

        RUST_SRC_PATH = "${pkgs.rust.packages.stable.rustPlatform.rustLibSrc}";
      };
    });
}

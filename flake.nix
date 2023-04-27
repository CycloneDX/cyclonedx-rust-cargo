{
  description = "A framework for developing the Rust CycloneDX implementation";

  inputs = {
    utils.url = "github:numtide/flake-utils";
    naersk.url = "github:nmattia/naersk";
  };

  outputs = { self, nixpkgs, utils, naersk }:
    utils.lib.eachDefaultSystem (system:
      let
        pkgs = nixpkgs.legacyPackages."${system}";
        naersk-lib = naersk.lib."${system}";
      in
      rec {
        packages.cyclonedx-rust-cargo = naersk-lib.buildPackage {
          pname = "cyclonedx-rust-cargo";
          root = ./.;
          doCheck = true;
          doDoc = true;
          doDocFail = true;

          buildInputs = with pkgs; [ openssl ];
          nativeBuildInputs = with pkgs; [ pkg-config ];
        };
        packages.default = packages.cyclonedx-rust-cargo;

        apps.cargo-cyclonedx = utils.lib.mkApp {
          drv = packages.cyclonedx-rust-cargo;
          name = "cargo-cyclonedx";
        };
        apps.default = apps.cargo-cyclonedx;

        devShells.default = pkgs.mkShell {
          packages = with pkgs; [
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

            # Nix tooling
            nixpkgs-fmt
          ];

          RUST_SRC_PATH = "${pkgs.rust.packages.stable.rustPlatform.rustLibSrc}";
        };
      });
}

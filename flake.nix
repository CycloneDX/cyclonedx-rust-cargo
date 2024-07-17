{
  description = "A framework for developing the Rust CycloneDX implementation";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";

    crane = {
      url = "github:ipetkov/crane";
      inputs.nixpkgs.follows = "nixpkgs";
    };

    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs = {
        nixpkgs.follows = "nixpkgs";
        flake-utils.follows = "flake-utils";
      };
    };
  };

  outputs = { self, nixpkgs, flake-utils, crane, rust-overlay, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs {
          inherit system;
          overlays = [ (import rust-overlay) ];
        };

        rustToolchain = pkgs.rust-bin.stable.latest.default.override {
          extensions = [ "rust-analyzer" "rust-src" ];
        };

        craneLib = (crane.mkLib pkgs).overrideToolchain rustToolchain;

        xmlFilter = path: _type: builtins.match ".*xml$" path != null;
        jsonFilter = path: _type: builtins.match ".*json$" path != null;
        snapshotTestFilter = path: _type: builtins.match ".*snap" path != null;
        stderrFilter = path: _type: builtins.match ".*stderr" path != null;

        srcFilter = path: type:
          (xmlFilter path type) || (jsonFilter path type) || (snapshotTestFilter path type) || (stderrFilter path type) || (craneLib.filterCargoSources path type);

        src = pkgs.lib.cleanSourceWith {
          src = craneLib.path ./.;
          filter = srcFilter;
        };

        commonArgs = {
          inherit src;

          pname = "cyclonedx-rust-cargo";
          version = "0.1.0";

          nativeBuildInputs = with pkgs; [ pkg-config ];
        };

        cargoArtifacts = craneLib.buildDepsOnly commonArgs;

        cyclonedx-rust-cargo = craneLib.buildPackage (commonArgs // {
          inherit cargoArtifacts;
        });
      in
      rec {
        checks = {
          inherit cyclonedx-rust-cargo;

          clippy = craneLib.cargoClippy (commonArgs // {
            inherit cargoArtifacts;
          });

          doc = craneLib.cargoDoc (commonArgs // {
            inherit cargoArtifacts;
          });

          fmt = craneLib.cargoFmt (commonArgs // {
            inherit src;
          });
        };

        packages.cyclonedx-rust-cargo = cyclonedx-rust-cargo;
        packages.default = packages.cyclonedx-rust-cargo;

        apps.cargo-cyclonedx = flake-utils.lib.mkApp {
          drv = packages.cyclonedx-rust-cargo;
          name = "cargo-cyclonedx";
        };
        apps.default = apps.cargo-cyclonedx;

        devShells.default = pkgs.mkShell {
          inputsFrom = builtins.attrValues self.checks.${system};

          packages = with pkgs; [
            rustToolchain
            cargo-edit
            cargo-msrv
            cargo-outdated

            # GitHub tooling
            gh

            # Nix tooling
            nixpkgs-fmt
          ];
        };
      });
}

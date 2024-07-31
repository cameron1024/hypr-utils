{
  description = "Build a cargo project with a custom toolchain";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";

    crane = {
      url = "github:ipetkov/crane";
      inputs.nixpkgs.follows = "nixpkgs";
    };

    flake-utils.url = "github:numtide/flake-utils";

    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = {
    self,
    nixpkgs,
    crane,
    flake-utils,
    rust-overlay,
    ...
  }:
    flake-utils.lib.eachDefaultSystem (system: let
      pkgs = import nixpkgs {
        inherit system;
        overlays = [(import rust-overlay)];
      };

      craneLib =
        (crane.mkLib pkgs).overrideToolchain (p:
          p.rust-bin.stable.latest.default);

      my-crate = craneLib.buildPackage {
        src = craneLib.cleanCargoSource ./.;
        strictDeps = true;
        nativeBuildInputs = with pkgs; [
          # sqlite
        ];
      };
    in {
      checks = {
        inherit my-crate;
      };

      packages.default = my-crate;

      devShells.default = craneLib.devShell {
        # Inherit inputs from checks.
        checks = self.checks.${system};

        # Extra inputs can be added here; cargo and rustc are provided by default
        # from the toolchain that was specified earlier.
        packages = [
        ];
      };
    });
}

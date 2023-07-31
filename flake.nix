{
  description = "Basic formatter for the Typst language with a future!";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";

    crane = {
      url = "github:ipetkov/crane";
      inputs.nixpkgs.follows = "nixpkgs";
    };

    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, crane, flake-utils, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs {
          inherit system;
        };

        craneLib = crane.lib.${system};
        typstfmt =
          craneLib.buildPackage
            {
              src = craneLib.path ./.;

              buildInputs = [
                pkgs.cargo-insta
              ];
            };
      in
      {
        checks = {
          inherit typstfmt;
        };

        packages.default = typstfmt;

        apps.default = flake-utils.lib.mkApp {
          drv = typstfmt;
        };

        devShells.default = pkgs.mkShell {
          inputsFrom = builtins.attrValues self.checks.${system};

          nativeBuildInputs = with pkgs; [
            cargo
            rustc
          ];
        };
      });
}

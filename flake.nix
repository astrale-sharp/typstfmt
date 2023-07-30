{
  description = "Basic formatter for the Typst language with a future!";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";

    flake-parts = {
      url = "github:hercules-ci/flake-parts";
      inputs.nixpkgs-lib.follows = "nixpkgs";
    };
  };

  outputs = inputs:
    inputs.flake-parts.lib.mkFlake { inherit inputs; } {
      systems = [ "x86_64-linux" ];

      perSystem =
        { config
        , pkgs
        , system
        , ...
        }: {
          devShells.default = pkgs.mkShell {
            inputsFrom = [ config.packages.typstfmt ];
            packages = with pkgs; [
              cargo
              clippy
              pre-commit
              rust-analyzer
              rustc
              rustfmt
              rustPackages.clippy
            ];

            RUST_SRC_PATH = pkgs.rustPlatform.rustLibSrc;
          };

          packages =
            {
              typstfmt = pkgs.rustPlatform.buildRustPackage {
                name = "typstfmt";

                src = ./.;

                cargoLock = {
                  lockFile = ./Cargo.lock;
                  outputHashes."typst-syntax-0.6.0" = "sha256-oAn783W7P8zY8gqPyy/w1goW/tdLJgf0qm2qAEJ9Vto=";
                };

                nativeBuildInputs = with pkgs; [ pkg-config cargo-insta ];

                hash = "sha256-oAn783W7P8zY8gqPyy/w1goW/tdLJgf0qm2qAEJ9Vto=";
              };
            }
            // { default = config.packages.typstfmt; };

          formatter = pkgs.alejandra;
        };
    };
}

{
  description = "rename-simple — rename files to clean, ASCII-safe slugs.";

  #----------------------------------------------------------------------------
  # FLAKE INPUTS
  #----------------------------------------------------------------------------
  #
  # Single input: nixpkgs. rename-simple is a pure-Rust crate with no foreign
  # runtime dependencies, so rustPlatform.buildRustPackage is sufficient.

  inputs.nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";

  #----------------------------------------------------------------------------
  # FLAKE OUTPUTS
  #----------------------------------------------------------------------------
  #
  # Consumed by the Darkone NixOS Framework (dnf) as a flake input:
  #   rename-simple.url = "github:darkone-linux/rename-simple";
  #   rename-simple.inputs.nixpkgs.follows = "nixpkgs";
  # then accessed as inputs.rename-simple.packages.${system}.default.
  #
  # Standalone use:
  #   nix build   — build the binary
  #   nix run     — run directly
  #   nix develop — enter the dev shell (same as nix-shell shell.nix)

  outputs =
    { self, nixpkgs }:
    let

      supportedSystems = [
        "x86_64-linux"
        "aarch64-linux"
      ];

      forAllSystems = nixpkgs.lib.genAttrs supportedSystems;

      # Version read from Cargo.toml: single source of truth for both
      # `cargo build` and the Nix derivation.
      cargoToml = builtins.fromTOML (builtins.readFile ./Cargo.toml);

    in
    {

      #------------------------------------------------------------------------
      # PACKAGES
      #------------------------------------------------------------------------

      packages = forAllSystems (
        system:
        let
          pkgs = nixpkgs.legacyPackages.${system};
          rename-simple = pkgs.rustPlatform.buildRustPackage {
            pname = "rename-simple";
            version = cargoToml.package.version;
            src = ./.;
            cargoLock.lockFile = ./Cargo.lock;

            # buildRustPackage installs binaries only. The hand-rolled
            # derivation this flake replaced also shipped the man page, so
            # restore it here or `just package nix` silently regresses.
            # Nix compresses man pages itself: install the uncompressed source.
            postInstall = ''
              install -Dm644 man/rename-simple.1 \
                $out/share/man/man1/rename-simple.1
            '';

            meta = {
              description = cargoToml.package.description;
              license = pkgs.lib.licenses.mit;
              mainProgram = "rename-simple";
            };
          };
        in
        {
          default = rename-simple;
          rename-simple = rename-simple;
        }
      );

      #------------------------------------------------------------------------
      # APPS
      #------------------------------------------------------------------------

      apps = forAllSystems (system: {
        default = {
          type = "app";
          program = "${self.packages.${system}.default}/bin/rename-simple";
        };
      });

      #------------------------------------------------------------------------
      # DEV SHELL
      #------------------------------------------------------------------------
      #
      # Delegates to shell.nix so both `nix-shell` and `nix develop` see the
      # same pinned toolchain.

      devShells = forAllSystems (
        system:
        let
          pkgs = nixpkgs.legacyPackages.${system};
        in
        {
          default = import ./shell.nix { inherit pkgs; };
        }
      );

    };
}

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

          # One recipe for two package sets: `pkgs` for the Nix package, and
          # `pkgs.pkgsStatic` (musl, fully static) for the binary shipped
          # outside Nix.
          mkRenameSimple =
            p:
            p.rustPlatform.buildRustPackage {
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
                license = p.lib.licenses.mit;
                mainProgram = "rename-simple";
              };
            };

          rename-simple = mkRenameSimple pkgs;
        in
        {
          default = rename-simple;
          rename-simple = rename-simple;

          # No program interpreter and no shared library: runs on any Linux
          # distribution of the same architecture. The GitHub release and the
          # .deb ship this one; the Nix package above links against the store.
          static = mkRenameSimple pkgs.pkgsStatic;
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
      # same pinned toolchain. Only `system` is passed: handing over this
      # flake's `nixpkgs` as `pkgs` would silently override shell.nix's pinned
      # revision, so `nix develop` would follow nixos-unstable while
      # `nix-shell shell.nix` stayed on the pin.

      devShells = forAllSystems (system: {
        default = import ./shell.nix { inherit system; };
      });

    };
}

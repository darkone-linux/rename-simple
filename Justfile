# rename-simple just file
# darkone@darkone.yt

set shell := ["bash", "-euo", "pipefail", "-c"]

version := `sed -n 's/^version = "\(.*\)"/\1/p' Cargo.toml`

# Justfile help
_default:
    @just --list

# Run full validation: format check, lint, and tests
test:
    cargo fmt --all --check \
        && cargo clippy --all-targets --all-features -- -D warnings \
        && cargo test

# Auto-fix clippy warnings and format code
clean:
    cargo clippy --fix --allow-dirty && cargo fmt

# Check that the project is clean (tests pass, git is not dirty)
_check_is_clean:
    cargo test -q \
        && git diff --quiet \
        && git diff --cached --quiet \
        || { echo "The project is not clean, all tests must pass and the git state must not be dirty." >&2; exit 1; }

# Build package for crates.io
package: _check_is_clean
    cargo package

# Build Debian .deb and NixOS packages
pkgs: _check_is_clean _pkgs_deb _pkgs_nixos

# Build Debian .deb package
_pkgs_deb:
    cargo build --release
    rm -rf target/pkgs/debian
    mkdir -p target/pkgs/debian/usr/bin \
             target/pkgs/debian/usr/share/man/man1 \
             target/pkgs/debian/DEBIAN
    cp target/release/rename-simple target/pkgs/debian/usr/bin/
    gzip -9cn < man/rename-simple.1 \
        > target/pkgs/debian/usr/share/man/man1/rename-simple.1.gz
    printf "%s\n" \
        "Package: rename-simple" \
        "Version: {{ version }}" \
        "Section: utils" \
        "Priority: optional" \
        "Architecture: amd64" \
        "Maintainer: darkone-linux" \
        "Description: Rename files to clean, ASCII-safe slugs" \
        " Renames files and/or directories by normalising accented" \
        " characters, replacing spaces and special chars with '-'," \
        " and lowercasing everything. Compound extensions (.tar.gz," \
        " .tar.bz2, .tar.xz, .tar.zst) are preserved. Hidden files" \
        " are never modified." \
        > target/pkgs/debian/DEBIAN/control
    cd target/pkgs/debian \
        && tar czf control.tar.gz DEBIAN/control \
        && tar czf data.tar.gz usr \
        && echo "2.0" > debian-binary \
        && ar rcs "rename-simple_{{ version }}_amd64.deb" \
            debian-binary control.tar.gz data.tar.gz
    rm -rf target/pkgs/debian/usr \
           target/pkgs/debian/DEBIAN \
           target/pkgs/debian/debian-binary \
           target/pkgs/debian/control.tar.gz \
           target/pkgs/debian/data.tar.gz

# Build NixOS package
_pkgs_nixos:
    cargo build --release
    rm -rf target/pkgs/nixos
    mkdir -p target/pkgs/nixos
    cp target/release/rename-simple target/pkgs/nixos/
    gzip -9cn < man/rename-simple.1 > target/pkgs/nixos/rename-simple.1.gz
    printf "%s\n" \
        "{ pkgs ? import <nixpkgs> {} }:" \
        "pkgs.stdenv.mkDerivation {" \
        "  pname = \"rename-simple\";" \
        "  version = \"{{ version }}\";" \
        "  dontUnpack = true;" \
        "  installPhase = ''" \
        "    mkdir -p \$out/bin \$out/share/man/man1" \
        "    cp \${./rename-simple} \$out/bin/" \
        "    cp \${./rename-simple.1.gz} \$out/share/man/man1/" \
        "  '';" \
        "}" \
        > target/pkgs/nixos/default.nix
    nixfmt target/pkgs/nixos/default.nix

# Deep clean: remove all build artifacts
mrproper:
    cargo clean

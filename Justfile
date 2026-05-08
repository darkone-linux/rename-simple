# rename-simple just file
# darkone@darkone.yt

set shell := ["bash", "-euo", "pipefail", "-c"]

version := `sed -n 's/^version = "\(.*\)"/\1/p' Cargo.toml`
arch := `dpkg --print-architecture 2>/dev/null || echo amd64`

# Justfile help
_default:
    @just --list

# ─── Validation & Quality Gate (matches AGENTS.md) ─────────────────────────

# Check that every source file is rustfmt-clean
[group('check')]
fmt-check:
    cargo fmt --all --check

# Run clippy with all targets/features and deny warnings
[group('check')]
lint:
    cargo clippy --all-targets --all-features -- -D warnings

# Run the full test suite
[group('check')]
unit:
    cargo test

# Audit dependencies for known vulnerabilities (RustSec advisory DB)
[group('check')]
audit:
    cargo audit

# Build the documentation to validate intra-doc links and docstrings
[group('check')]
doc:
    cargo doc --no-deps

# Full validation gate: format check, lint, tests, audit, docs
[group('check')]
test: fmt-check lint unit audit doc

# Alias used by CI: same as `test` (full quality gate)
[group('check')]
ci: test

# ─── Day-to-day helpers ────────────────────────────────────────────────────

# Auto-fix clippy warnings and reformat code
[group('daily')]
fix:
    cargo clippy --all-targets --all-features --fix --allow-dirty
    cargo fmt --all

# Install the binary into ~/.cargo/bin
[group('daily')]
install:
    cargo install --path .

# Remove the installed binary from ~/.cargo/bin
[group('daily')]
uninstall:
    cargo uninstall rename-simple

# Preview the man page in a pager
[group('daily')]
man:
    man -l man/rename-simple.1

# ─── Release / packaging ───────────────────────────────────────────────────

# Check that the project is clean (tests pass, git is not dirty)
_check_is_clean:
    cargo test -q
    if [ -n "$(git status --porcelain)" ]; then \
        echo "The project is not clean: working tree has uncommitted or untracked files." >&2; \
        exit 1; \
    fi

# Bump the version number in Cargo.toml (patch | minor | major)
[group('packaging')]
bump level="patch":
    #!/usr/bin/env bash
    set -euo pipefail
    current=$(sed -n 's/^version = "\(.*\)"/\1/p' Cargo.toml)
    IFS='.' read -r major minor patch <<< "$current"
    case "{{ level }}" in
        major) major=$((major + 1)); minor=0; patch=0 ;;
        minor) minor=$((minor + 1)); patch=0 ;;
        patch) patch=$((patch + 1)) ;;
        *) echo "Usage: just bump [patch|minor|major]" >&2; exit 1 ;;
    esac
    new="${major}.${minor}.${patch}"
    sed -i "s/^version = \"${current}\"/version = \"${new}\"/" Cargo.toml
    cargo check -q
    echo "Version bumped: ${current} → ${new}"
    echo "Next step: update CHANGELOG.md, then run: just release"

# Full release: test → commit → tag → push → GitHub Release → cargo publish
[group('packaging')]
release:
    #!/usr/bin/env bash
    set -euo pipefail
    v=$(sed -n 's/^version = "\(.*\)"/\1/p' Cargo.toml)
    tag="v${v}"

    # Guard: CHANGELOG.md must have an entry for this version
    if ! grep -q "## \[${v}\]" CHANGELOG.md; then
        echo "Error: CHANGELOG.md has no entry for [${v}]. Update it first." >&2; exit 1
    fi

    # Full quality gate (always run)
    just test

    # Commit Cargo.toml, Cargo.lock and CHANGELOG.md if anything is pending
    git add Cargo.toml Cargo.lock CHANGELOG.md
    git diff --cached --quiet || git commit -m "chore: release ${tag}"

    # Annotated tag — only if not already present
    if git rev-parse "$tag" >/dev/null 2>&1; then
        echo "Tag $tag already exists — skipping tag creation."
    else
        git tag -a "$tag" -m "Release ${tag}"
    fi

    # Push branch + tags (idempotent: no-op when already up-to-date)
    git push origin main --follow-tags

    # GitHub Release — skip if already exists
    if gh release view "$tag" >/dev/null 2>&1; then
        echo "GitHub release $tag already exists — skipping."
    else
        notes=$(awk "/^## \[${v}\]/{found=1; next} found && /^## \[/{exit} found{print}" CHANGELOG.md)
        gh release create "$tag" --title "$tag" --notes "$notes"
        echo "GitHub release $tag created."
    fi

    # Publish to crates.io — skip if this version is already published
    if curl -sf "https://crates.io/api/v1/crates/rename-simple/${v}" >/dev/null 2>&1; then
        echo "crates.io rename-simple ${v} already published — skipping."
    else
        cargo publish
        echo "crates.io rename-simple ${v} published."
    fi

    echo "Release ${tag} complete ✓"

# Build package for crates.io
[group('packaging')]
package: _check_is_clean
    cargo package

# Build Debian .deb and NixOS packages
[group('packaging')]
pkgs: _check_is_clean _pkgs_deb _pkgs_nixos

# Build Debian .deb package
_pkgs_deb:
    cargo build --release
    rm -rf target/pkgs/debian
    mkdir -p target/pkgs/debian/usr/bin \
             target/pkgs/debian/usr/share/man/man1 \
             target/pkgs/debian/DEBIAN
    cp target/release/rename-simple target/pkgs/debian/usr/bin/
    chmod 0755 target/pkgs/debian/usr/bin/rename-simple
    gzip -9cn < man/rename-simple.1 \
        > target/pkgs/debian/usr/share/man/man1/rename-simple.1.gz
    INSTALLED_SIZE=$(du -ks target/pkgs/debian/usr | cut -f1); \
    printf "%s\n" \
        "Package: rename-simple" \
        "Version: {{ version }}" \
        "Section: utils" \
        "Priority: optional" \
        "Architecture: {{ arch }}" \
        "Maintainer: darkone-linux" \
        "Homepage: https://github.com/darkone-linux/rename-simple" \
        "Installed-Size: $INSTALLED_SIZE" \
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
        && ar rcs "rename-simple_{{ version }}_{{ arch }}.deb" \
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

# ─── Cleanup ───────────────────────────────────────────────────────────────

# Deep clean: remove all build artifacts (cargo + packaging output)
[group('daily')]
clean:
    cargo clean
    rm -rf target/pkgs

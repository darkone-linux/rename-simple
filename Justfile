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

# Full release: test → commit → tag → push → package → GitHub Release → cargo publish
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

    # Build binary and .deb for GitHub release assets
    just package all

    # GitHub Release — skip if already exists; upload assets idempotently
    if gh release view "$tag" >/dev/null 2>&1; then
        echo "GitHub release $tag already exists — checking assets."
        if ! gh release view "$tag" --json assets -q '.assets[].name' \
            | grep -qx 'rename-simple'; then
            gh release upload "$tag" target/release/rename-simple
            echo "  → uploaded rename-simple binary"
        else
            echo "  → rename-simple binary already present — skipping"
        fi
        deb_file="rename-simple_{{ version }}_{{ arch }}.deb"
        if ! gh release view "$tag" --json assets -q '.assets[].name' \
            | grep -qx "$deb_file"; then
            gh release upload "$tag" "target/pkgs/debian/$deb_file"
            echo "  → uploaded $deb_file"
        else
            echo "  → $deb_file already present — skipping"
        fi
    else
        notes=$(awk "/^## \[${v}\]/{found=1; next} found && /^## \[/{exit} found{print}" CHANGELOG.md)
        gh release create "$tag" --title "$tag" --notes "$notes" \
            target/release/rename-simple \
            "target/pkgs/debian/rename-simple_{{ version }}_{{ arch }}.deb"
        echo "GitHub release $tag created with artifacts."
    fi

    # Publish to crates.io — skip if this version is already published
    if curl -sf "https://crates.io/api/v1/crates/rename-simple/${v}" >/dev/null 2>&1; then
        echo "crates.io rename-simple ${v} already published — skipping."
    else
        cargo publish
        echo "crates.io rename-simple ${v} published."
    fi

    echo "Release ${tag} complete ✓"

# Build packages: cargo (crates.io), deb (Debian .deb), nix (NixOS), all (default)
[group('packaging')]
package type="all": _check_is_clean
    #!/usr/bin/env bash
    set -euo pipefail
    case "{{ type }}" in
        all)
            cargo package
            just _pkgs_deb
            just _pkgs_nix
            ;;
        cargo)
            cargo package
            ;;
        deb)
            just _pkgs_deb
            ;;
        nix)
            just _pkgs_nix
            ;;
        *)
            echo "Usage: just package [all|cargo|deb|nix]" >&2
            exit 1
            ;;
    esac

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

# Build and verify the Nix flake package
_pkgs_nix:
    nix build .#rename-simple

# ─── Cleanup ───────────────────────────────────────────────────────────────

# Deep clean: remove all build artifacts (cargo + packaging output)
[group('daily')]
clean:
    cargo clean
    rm -rf target/pkgs

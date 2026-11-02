# rename-simple just file
# darkone@darkone.yt

set shell := ["bash", "-euo", "pipefail", "-c"]

version := `sed -n 's/^version = "\(.*\)"/\1/p' Cargo.toml`
# Debian architecture of this machine, which is also the one `nix build .#static`
# targets; without dpkg, map the kernel's name instead of assuming amd64.
arch := `dpkg --print-architecture 2>/dev/null || case "$(uname -m)" in x86_64) echo amd64 ;; aarch64) echo arm64 ;; *) uname -m ;; esac`

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

# Check that the working tree is clean (no uncommitted or untracked files).
# Testing is the job of `just test` / `just unit`, not of this guard.
_check_is_clean:
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

# Full release: test → commit → tag → package → push → GitHub Release → cargo publish
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

    # Build packages (binary, .deb, …) BEFORE anything leaves the machine, so a
    # packaging failure stays local and fully recoverable — nothing has been
    # pushed or published yet at this point.
    just package all

    # Push branch + tags (idempotent: no-op when already up-to-date)
    git push origin main --follow-tags

    # GitHub Release — skip if already exists; upload assets idempotently
    if gh release view "$tag" >/dev/null 2>&1; then
        echo "GitHub release $tag already exists — checking assets."
        if ! gh release view "$tag" --json assets -q '.assets[].name' \
            | grep -qx 'rename-simple'; then
            gh release upload "$tag" target/pkgs/rename-simple
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
            target/pkgs/rename-simple \
            "target/pkgs/debian/rename-simple_{{ version }}_{{ arch }}.deb"
        echo "GitHub release $tag created with artifacts."
    fi

    # Publish to crates.io — skip if this version is already published.
    # crates.io rejects requests without a User-Agent (HTTP 403), so the guard
    # must send one; otherwise it always falls through to `cargo publish`, which
    # then errors on an already-published version instead of skipping cleanly.
    ua="rename-simple-release (https://github.com/darkone-linux/rename-simple)"
    if curl -sf -A "$ua" "https://crates.io/api/v1/crates/rename-simple/${v}" >/dev/null 2>&1; then
        echo "crates.io rename-simple ${v} already published — skipping."
    else
        cargo publish
        echo "crates.io rename-simple ${v} published."
    fi

    echo "Release ${tag} complete ✓"

# Build packages: cargo (crates.io), bin (static binary), deb (Debian .deb), nix (NixOS), all (default)
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
        bin)
            just _pkgs_bin
            ;;
        deb)
            just _pkgs_deb
            ;;
        nix)
            just _pkgs_nix
            ;;
        *)
            echo "Usage: just package [all|cargo|bin|deb|nix]" >&2
            exit 1
            ;;
    esac

# Fail unless `bin` is static: a binary shipped to other distributions must not
# need a program interpreter or shared libraries, which a dev-shell build loads
# from /nix/store (and looks up in a RUNPATH that names a home directory).
_check_portable bin:
    #!/usr/bin/env bash
    set -euo pipefail
    interp=$(LC_ALL=C readelf -lW "{{ bin }}" | sed -n 's/.*Requesting program interpreter: \(.*\)]$/\1/p')
    if [ -n "$interp" ]; then
        echo "error: {{ bin }} needs the program interpreter $interp: ship a static binary" >&2
        exit 1
    fi
    dynamic=$(LC_ALL=C readelf -dW "{{ bin }}" 2>/dev/null || true)
    if grep -q '(NEEDED)' <<< "$dynamic"; then
        echo "error: {{ bin }} loads shared libraries:" >&2
        grep '(NEEDED)' <<< "$dynamic" >&2
        exit 1
    fi
    # An empty RUNPATH (left by patchelf --shrink-rpath) points nowhere.
    if grep -E '\((RUNPATH|RPATH)\)' <<< "$dynamic" | grep -qv '\[\]$'; then
        echo "error: {{ bin }} has a library search path:" >&2
        grep -E '\((RUNPATH|RPATH)\)' <<< "$dynamic" >&2
        exit 1
    fi
    echo "{{ bin }}: static, no interpreter, no shared library"

# Build the static binary shipped outside Nix (GitHub release, .deb) into
# target/pkgs/rename-simple. `cargo build` inside the dev shell would link it
# against /nix/store, which other distributions do not have.
_pkgs_bin:
    mkdir -p target/pkgs
    install -m 0755 "$(nix build .#static --no-link --print-out-paths)/bin/rename-simple" \
        target/pkgs/rename-simple
    just _check_portable target/pkgs/rename-simple

# Check a .deb the way dpkg sees it: readable control fields, every entry owned
# by root with sane modes, a static binary, and a real install into a scratch
# root (`dpkg --root`, no privilege needed).
_check_deb deb:
    #!/usr/bin/env bash
    set -euo pipefail
    deb=$(realpath "{{ deb }}")
    dpkg-deb --field "$deb" Package Version Architecture Maintainer
    contents=$(dpkg-deb --contents "$deb")
    bad=$(awk '$2 != "root/root"' <<< "$contents")
    if [ -n "$bad" ]; then
        echo "error: $deb has entries not owned by root:" >&2
        echo "$bad" >&2
        exit 1
    fi
    bad=$(awk '($1 ~ /^d/ && $1 != "drwxr-xr-x") ||
               ($1 ~ /^-/ && $6 ~ /^\.\/usr\/bin\// && $1 != "-rwxr-xr-x") ||
               ($1 ~ /^-/ && $6 !~ /^\.\/usr\/bin\// && $1 != "-rw-r--r--")' <<< "$contents")
    if [ -n "$bad" ]; then
        echo "error: $deb has unexpected modes:" >&2
        echo "$bad" >&2
        exit 1
    fi
    tmp=$(mktemp -d)
    trap 'rm -rf "$tmp"' EXIT
    mkdir -p "$tmp/var/lib/dpkg/updates" "$tmp/var/lib/dpkg/info" "$tmp/var/lib/dpkg/triggers"
    touch "$tmp/var/lib/dpkg/status" "$tmp/var/lib/dpkg/available"
    dpkg --force-not-root --force-script-chrootless --root="$tmp" --install "$deb"
    just _check_portable "$tmp/usr/bin/rename-simple"
    "$tmp/usr/bin/rename-simple" --version
    echo "$deb: valid, root-owned, installs with dpkg"

# Build Debian .deb package (static binary + man page), then check it
_pkgs_deb: _pkgs_bin
    #!/usr/bin/env bash
    set -euo pipefail
    deb="target/pkgs/debian/rename-simple_{{ version }}_{{ arch }}.deb"
    root=target/pkgs/debian/root
    rm -rf target/pkgs/debian
    # Every mode is explicit: the package must not depend on the builder's umask.
    install -d -m 0755 "$root" "$root/DEBIAN" "$root/usr" "$root/usr/bin" \
        "$root/usr/share" "$root/usr/share/man" "$root/usr/share/man/man1"
    install -m 0755 target/pkgs/rename-simple "$root/usr/bin/rename-simple"
    gzip -9cn < man/rename-simple.1 > "$root/usr/share/man/man1/rename-simple.1.gz"
    chmod 0644 "$root/usr/share/man/man1/rename-simple.1.gz"
    installed_size=$(du -ks "$root/usr" | cut -f1)
    printf "%s\n" \
        "Package: rename-simple" \
        "Version: {{ version }}" \
        "Section: utils" \
        "Priority: optional" \
        "Architecture: {{ arch }}" \
        "Maintainer: Guillaume Ponçon <darkone@darkone.yt>" \
        "Homepage: https://github.com/darkone-linux/rename-simple" \
        "Installed-Size: $installed_size" \
        "Description: Rename files to clean, ASCII-safe slugs" \
        " Renames files and/or directories by normalising accented" \
        " characters, replacing spaces and special chars with '-'," \
        " and lowercasing everything. Compound extensions (.tar.gz," \
        " .tar.bz2, .tar.xz, .tar.zst) are preserved. Hidden files" \
        " are never modified." \
        > "$root/DEBIAN/control"
    chmod 0644 "$root/DEBIAN/control"
    # dpkg-deb lays the archive out the way dpkg reads it (`./control` at the
    # root of the control archive), and --root-owner-group records every entry
    # as root:root instead of the builder's uid, which would otherwise own
    # /usr/bin/rename-simple on the target system.
    dpkg-deb --root-owner-group --build "$root" "$deb"
    rm -rf "$root"
    just _check_deb "$deb"

# Build and verify the Nix flake package
_pkgs_nix:
    nix build .#rename-simple

# ─── Cleanup ───────────────────────────────────────────────────────────────

# Deep clean: remove all build artifacts (cargo + packaging output)
[group('daily')]
clean:
    cargo clean
    rm -rf target/pkgs

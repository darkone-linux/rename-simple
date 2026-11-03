# Tech Notes

## Release procedure

### Prerequisites (one-time setup)

```bash
cargo login        # crates.io token
gh auth login      # GitHub token
```

### Steps

```bash
# 1. Bump the version (patch | minor | major)
just bump patch

# 2. Update CHANGELOG.md — add a [x.y.z] section at the top

# 3. Publish
just release
```

`just release` runs the full quality gate (`just test`), commits
`Cargo.toml`, `Cargo.lock` and `CHANGELOG.md`, creates an annotated git
tag, builds the packages (`just package all`), pushes to GitHub, creates a
GitHub Release (notes pulled from `CHANGELOG.md`, static binary and `.deb`
attached), and publishes to crates.io.

### Release binary

The binary attached to the GitHub Release and packed in the `.deb` is the
fully static musl build of the flake, `nix build .#static` (`just package
bin` copies it to `target/pkgs/rename-simple`). A `cargo build` inside the
dev shell links against `/nix/store` and does not run on other
distributions: `just _check_portable <binary>` rejects such a binary.

`just package deb` builds the `.deb` with `dpkg-deb --root-owner-group`, then
`just _check_deb <file>` checks it the way dpkg sees it: readable control
fields, every entry owned by root with the expected modes, a static binary,
and a `dpkg --install` into a scratch root.

### Guards and reruns

`just release` stops before anything leaves the machine if:
- `CHANGELOG.md` has no entry for the current version (no publishing without
  release notes);
- the quality gate (`just test`) or the packaging (`just package all`) fails;
- the tag already exists on another commit than `HEAD` (the packages built
  from `HEAD` would not match the tagged code).

Every other step is skipped when already done: the commit when nothing is
pending, the tag when it marks `HEAD`, the GitHub Release and each of its
assets when present, the crates.io upload when the version is published.
After a partial failure (network, authentication), fix the cause and rerun
`just release`: it resumes where it stopped.

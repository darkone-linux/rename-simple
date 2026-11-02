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
tag, pushes to GitHub, creates a GitHub Release (notes pulled from
`CHANGELOG.md`), and publishes to crates.io.

### Release binary

The binary attached to the GitHub Release and packed in the `.deb` is the
fully static musl build of the flake, `nix build .#static` (`just package
bin` copies it to `target/pkgs/rename-simple`). A `cargo build` inside the
dev shell links against `/nix/store` and does not run on other
distributions: `just _check_portable <binary>` rejects such a binary.

### Guards

`just release` aborts early if:
- the git tag already exists (prevents accidental double-publish)
- `CHANGELOG.md` has no entry for the current version (prevents publishing without release notes)

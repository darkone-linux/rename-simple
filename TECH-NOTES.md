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

### Guards

`just release` aborts early if:
- the git tag already exists (prevents accidental double-publish)
- `CHANGELOG.md` has no entry for the current version (prevents publishing without release notes)

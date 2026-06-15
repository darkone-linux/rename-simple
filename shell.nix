{
  # Pinned nixpkgs so local dev shells and CI resolve the exact same toolchain
  # (cargo, clippy, rustfmt…). Using the ambient `<nixpkgs>` channel let CI
  # drift ahead of local checkouts, surfacing clippy lints that `just test`
  # could not see locally.
  #
  # To update: bump `rev`, then run
  #   nix-prefetch-url --unpack https://github.com/NixOS/nixpkgs/archive/<rev>.tar.gz
  # and paste the printed hash into `sha256`.
  pkgs ? import (fetchTarball {
    url = "https://github.com/NixOS/nixpkgs/archive/c6d65881c5624c9cae5ea6cedef24699b0c0a4c0.tar.gz";
    sha256 = "1yf4qv3scjygdkg67nibrhbddg3154mv9cxffvykmwcrwfcrrlaq";
  }) { },
}:

pkgs.mkShell {
  buildInputs = with pkgs; [
    cargo
    cargo-audit
    clippy
    gh
    just
    nixfmt
    pkg-config
    rust-analyzer
    rustc
    rustfmt
  ];

  shellHook = ''
    alias build="cargo build --release"
    alias test="cargo test"

    # Keep the user's interactive shell (e.g. zsh) instead of falling back to bash.
    if [ -n "$SHELL" ] && [ "$(basename "$SHELL")" != "bash" ] && [ -z "$IN_NIX_SHELL_USER_SHELL" ]; then
      export IN_NIX_SHELL_USER_SHELL=1
      exec "$SHELL"
    fi
  '';

  RUST_BACKTRACE = 1;
}

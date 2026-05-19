{
  pkgs ? import <nixpkgs> { },
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

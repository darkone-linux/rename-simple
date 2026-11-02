{
  # Pinned nixpkgs so local dev shells and CI resolve the exact same toolchain
  # (cargo, clippy, rustfmt…). Using the ambient `<nixpkgs>` channel let CI
  # drift ahead of local checkouts, surfacing clippy lints that `just test`
  # could not see locally.
  #
  # To update: bump `rev`, then run
  #   nix-prefetch-url --unpack https://github.com/NixOS/nixpkgs/archive/<rev>.tar.gz
  # and paste the printed hash into `sha256`.
  #
  # `system` is a parameter so flake.nix can select the target without having
  # to supply its own `pkgs` — passing `pkgs` would override the pin above,
  # which is precisely the drift this file exists to prevent. Under a flake,
  # evaluation is pure and `builtins.currentSystem` is unavailable, hence the
  # explicit argument.
  system ? builtins.currentSystem,
  pkgs ? import (fetchTarball {
    url = "https://github.com/NixOS/nixpkgs/archive/0bb2eedc20e4739f29e8045034f2828ad7054f8f.tar.gz";
    sha256 = "008j4dj5yrzww91qvp1lp2c9d1lkwc6i2dpainf5nkdkr3rwj8vy";
  }) { inherit system; },
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

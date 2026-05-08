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
    pkg-config
    rust-analyzer
    rustc
    rustfmt
  ];

  shellHook = ''
    alias build="cargo build --release"
    alias test="cargo test"
  '';

  RUST_BACKTRACE = 1;
}

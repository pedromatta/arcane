{ pkgs ? import <nixpkgs> {} }:

pkgs.mkShell {
  buildInputs = with pkgs; [
    rustc
    cargo
    rustfmt
    clippy
    pkg-config
    openssl
    sqlite
  ];

  shellHook = ''
    echo "Welcome to the Arcane development environment!"
  '';
}

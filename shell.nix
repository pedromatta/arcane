{ pkgs ? import <nixpkgs> {} }:

pkgs.mkShell {

  nativeBuildInputs = with pkgs; [
    pkg-config
  ];

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

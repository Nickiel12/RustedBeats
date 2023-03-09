{ pkgs ? import <nixpkgs> {}}:

let
  rust_overlay = import (builtins.fetchTarball "https://github.com/oxalica/rust-overlay/archive/master.tar.gz");
  pkgs = import <nixpkgs> { overlays = [ rust_overlay ]; };
  ruststable = (pkgs.latest.rustChannels.stable.default.override {
      extensions = [
        "rust-src"
      ];
    });
in
pkgs.mkShell {
  buildInputs = with pkgs; [
    ruststable
    sqlite
    sqliteman
    pkg-config
    alsa-lib
  ];

  RUST_BACKTRACE = 1;

   shellHook = ''
  cargo install --locked bacon
  export PATH=$HOME/.cargo/bin:$PATH
'';
}

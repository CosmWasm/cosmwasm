{ pkgs ? import <nixpkgs> { overlays = [ (import (builtins.fetchTarball "https://github.com/oxalica/rust-overlay/archive/master.tar.gz")) ]; } }:
let x = 42; in pkgs.mkShell {
  nativeBuildInputs = [ (pkgs.rust-bin.fromRustupToolchainFile ./rust-toolchain.toml) ];
  #
}

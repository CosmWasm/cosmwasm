{ pkgs ? import <nixpkgs> { overlays = [ (import (builtins.fetchTarball "https://github.com/oxalica/rust-overlay/archive/refs/heads/stable.zip")) ]; } }:
let
  rust-as-on-ci = pkgs.rust-bin.fromRustupToolchainFile ./rust-toolchain.toml;
  features = " --features abort,iterator,staking,stargate,cosmwasm_1_2";
  package="cosmwasm-std";
  check-no-std = pkgs.writeShellApplication rec {
    name = "check-no-std";
    runtimeInputs = [ rust-as-on-ci ];
    text = ''
      cargo build --locked --no-default-features --target thumbv7em-none-eabi --package ${package} ${features}
    '';
  };
  check-wasm-std = pkgs.writeShellApplication rec {
    name = "check-wasm-std";
    runtimeInputs = [ rust-as-on-ci ];
    text = ''
      cargo build --target wasm32-unknown-unknown --locked ${features},std --package ${package}
    '';
  };
in
pkgs.mkShell {
  nativeBuildInputs = [ rust-as-on-ci check-no-std check-wasm-std];
}

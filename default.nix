{ pkgs, ... }:
let
  manifest = (pkgs.lib.importTOML ./Cargo.toml).package;
in
pkgs.rustPlatform.buildRustPackage rec {
  pname = manifest.name;
  version = manifest.version;
  description = manifest.description;
  license = manifest.license;
  homepage = manifest.repository;
  cargoLock.lockFile = ./Cargo.lock;
  src = pkgs.lib.cleanSource ./.;
  checkFlags = [
    # We skip these tests because they rely on creating containers to simulate
    # IMAP functionality
    "--skip=imap_tests"
  ];
  nativeBuildInputs = with pkgs; [
    pkg-config
  ];
  buildInputs = with pkgs; [
    openssl
    sqlite
  ];
}

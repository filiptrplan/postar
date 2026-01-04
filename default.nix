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
    installShellFiles
  ];
  buildInputs = with pkgs; [
    openssl
    sqlite
  ];
  postInstall = ''
    find . -name "*.1" -type f | while read -r file; do installManPage "$file"; done

    find . -name "*.bash" -type f | while read -r file; do installShellCompletion --bash "$file"; done
    find . -name "*.fish" -type f | while read -r file; do installShellCompletion --fish "$file"; done
    find . -name "_*" -type f | while read -r file; do installShellCompletion --zsh "$file"; done
  '';
}

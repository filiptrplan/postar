{
  description = "An email filtering program an service";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
    rust-overlay = {
      inputs.nixpkgs.follows = "nixpkgs";
    };
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs =
    {
      self,
      nixpkgs,
      rust-overlay,
      flake-utils,
      ...
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs {
          inherit system overlays;
        };
      in
      {
        packages.default = pkgs.callPackage ./default.nix { };
        devShells.default =
          with pkgs;
          mkShell {
            buildInputs = [
              mailutils
              just
              openssl
              pkg-config
              eza
              fd
              python3
              rust-bin.stable.latest.default
              rust-analyzer
              sqlite
            ];
          };
      }
    )
    // {
      homeManagerModules.default = import ./hm-module.nix { inherit self; };
      homeManagerModules.postar = self.homeManagerModules.default;
    };
}

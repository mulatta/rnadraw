{
  description = "rnadraw";

  inputs = {
    # keep-sorted start
    crane.url = "github:ipetkov/crane";
    flake-parts.inputs.nixpkgs-lib.follows = "nixpkgs";
    flake-parts.url = "github:hercules-ci/flake-parts";
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay.inputs.nixpkgs.follows = "nixpkgs";
    rust-overlay.url = "github:oxalica/rust-overlay";
    systems.url = "github:nix-systems/default";
    treefmt-nix.inputs.nixpkgs.follows = "nixpkgs";
    treefmt-nix.url = "github:numtide/treefmt-nix";
    # keep-sorted end
  };

  nixConfig = {
    extra-substituters = [ "https://cache.mulatta.io" ];
    extra-trusted-public-keys = [ "cache.mulatta.io-1:DrV+Oy2azNyVKM7ihhD1QoOetRUnW+1G6RWToUpSO4U=" ];
  };

  outputs =
    inputs@{ flake-parts, ... }:
    flake-parts.lib.mkFlake { inherit inputs; } {
      systems = import inputs.systems;
      imports = [
        ./nix/checks.nix
        ./nix/formatter.nix
        ./nix/packages.nix
        ./nix/shell.nix
      ];

      perSystem =
        { system, ... }:
        let
          pkgs = import inputs.nixpkgs {
            inherit system;
            overlays = [ (import inputs.rust-overlay) ];
          };
          rustToolchain = pkgs.rust-bin.stable.latest.default.override {
            targets = [
              "wasm32-unknown-unknown"
              "wasm32-wasip1"
            ];
          };
        in
        {
          _module.args = {
            inherit pkgs;
            craneLib = (inputs.crane.mkLib pkgs).overrideToolchain rustToolchain;
          };
        };
    };
}

{
  description = "rnadraw";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
    rust-overlay.inputs.nixpkgs.follows = "nixpkgs";
    treefmt-nix.url = "github:numtide/treefmt-nix";
    treefmt-nix.inputs.nixpkgs.follows = "nixpkgs";
  };

  nixConfig = {
    allow-import-from-derivation = false;
    extra-substituters = [ "https://cache.mulatta.io" ];
    extra-trusted-public-keys = [ "cache.mulatta.io-1:DrV+Oy2azNyVKM7ihhD1QoOetRUnW+1G6RWToUpSO4U=" ];
  };

  outputs =
    inputs@{ self, nixpkgs, ... }:
    let
      inherit (nixpkgs) lib;

      systems = [
        "x86_64-linux"
        "aarch64-linux"
        "aarch64-darwin"
      ];
      eachSystem = lib.genAttrs systems;

      flake = self // {
        inherit inputs;
      };

      wasmBindgenDependency =
        (builtins.fromTOML (builtins.readFile ./crates/wasm/Cargo.toml)).dependencies."wasm-bindgen";
      wasmBindgenVersion = lib.removePrefix "=" (
        if builtins.isAttrs wasmBindgenDependency then
          wasmBindgenDependency.version
        else
          wasmBindgenDependency
      );
      wasmBindgenCliFor =
        pkgs: pkgs.${"wasm-bindgen-cli_${lib.replaceStrings [ "." ] [ "_" ] wasmBindgenVersion}"};

      pkgsFor = eachSystem (
        system:
        import nixpkgs {
          inherit system;
          overlays = [ inputs.rust-overlay.overlays.default ];
        }
      );

      mkPackagesFor =
        pkgs:
        let
          rustToolchain = pkgs.rust-bin.stable.latest.default.override {
            targets = [
              "wasm32-unknown-unknown"
              "wasm32-wasip1"
            ];
          };

          scope = lib.makeScope pkgs.newScope (self: {
            inherit flake inputs;
            source.src = flake;

            rustPlatform = pkgs.makeRustPlatform {
              cargo = rustToolchain;
              rustc = rustToolchain;
            };
            rnadraw-wasm-raw = self.callPackage ./nix/packages/wasm/raw.nix { };

            cli = self.callPackage ./nix/packages/cli/package.nix { };
            formatter = self.callPackage ./nix/packages/formatter/package.nix { inherit pkgs; };
            wasi = self.callPackage ./nix/packages/wasi/package.nix { };
            wasm = self.callPackage ./nix/packages/wasm/package.nix {
              wasm-bindgen-cli = wasmBindgenCliFor pkgs;
            };
          });
        in
        {
          inherit (scope)
            cli
            formatter
            wasi
            wasm
            ;
          default = scope.cli;
        };

      packages = eachSystem (system: mkPackagesFor pkgsFor.${system});
    in
    {
      inherit packages;

      checks = eachSystem (
        system:
        import ./nix/checks.nix {
          packages = packages.${system};
        }
        // {
          devshell-default = self.devShells.${system}.default;
        }
      );

      devShells = eachSystem (
        system:
        import ./nix/shell.nix {
          pkgs = pkgsFor.${system};
          formatter = packages.${system}.formatter;
          wasm-bindgen-cli = wasmBindgenCliFor pkgsFor.${system};
        }
      );

      formatter = eachSystem (system: packages.${system}.formatter);
    };
}

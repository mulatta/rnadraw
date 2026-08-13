{
  description = "rnadraw";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
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

      pkgsFor = eachSystem (system: import nixpkgs { inherit system; });

      mkPackagesFor =
        pkgs:
        let
          scope = lib.makeScope pkgs.newScope (self: {
            inherit flake inputs;
            source.src = flake;

            rnadraw-wasm-raw = self.callPackage ./nix/packages/wasm/raw.nix { };

            cli = self.callPackage ./nix/packages/cli/package.nix { };
            formatter = self.callPackage ./nix/packages/formatter/package.nix { inherit pkgs; };
            wasi = self.callPackage ./nix/packages/wasi/package.nix { };
            wasm = self.callPackage ./nix/packages/wasm/package.nix { };
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
        }
      );

      formatter = eachSystem (system: packages.${system}.formatter);
    };
}

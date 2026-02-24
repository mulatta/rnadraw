{ inputs, ... }:
{
  perSystem =
    {
      pkgs,
      ...
    }:
    let
      # Rust toolchain: stable with WASM/WASI targets
      rustToolchain = pkgs.rust-bin.stable.latest.default.override {
        targets = [
          "wasm32-unknown-unknown"
          "wasm32-wasip1"
        ];
      };

      craneLib = (inputs.crane.mkLib pkgs).overrideToolchain rustToolchain;

      # Common source filtering: only Rust/TOML/JSON files
      src = pkgs.lib.cleanSourceWith {
        src = inputs.self;
        filter =
          path: type: (craneLib.filterCargoSources path type) || (builtins.match ".*\\.json$" path != null);
      };

      # --- Native (CLI) ---
      commonArgs = {
        inherit src;
        strictDeps = true;
      };

      cargoArtifacts = craneLib.buildDepsOnly commonArgs;

      cli = craneLib.buildPackage (
        commonArgs
        // {
          inherit cargoArtifacts;
          cargoExtraArgs = "-p rnadraw";
        }
      );

      # --- WASM (browser, wasm-bindgen) ---
      wasmArgs = {
        inherit src;
        strictDeps = true;
        doCheck = false;
        CARGO_BUILD_TARGET = "wasm32-unknown-unknown";
        cargoExtraArgs = "-p rnadraw-wasm";
      };

      wasmArtifacts = craneLib.buildDepsOnly wasmArgs;

      wasmRaw = craneLib.buildPackage (
        wasmArgs
        // {
          cargoArtifacts = wasmArtifacts;
        }
      );

      wasm = pkgs.stdenv.mkDerivation {
        pname = "rnadraw-wasm";
        inherit (wasmRaw) version;
        nativeBuildInputs = [ pkgs.wasm-bindgen-cli ];
        dontUnpack = true;
        buildPhase = ''
          wasm-bindgen \
            --target web \
            --out-dir $out \
            ${wasmRaw}/lib/rnadraw_wasm.wasm
        '';
        dontInstall = true;
      };

      # --- WASI (server/portable CLI) ---
      wasiArgs = {
        inherit src;
        strictDeps = true;
        doCheck = false;
        CARGO_BUILD_TARGET = "wasm32-wasip1";
        cargoExtraArgs = "-p rnadraw";
      };

      wasiArtifacts = craneLib.buildDepsOnly wasiArgs;

      wasi = craneLib.buildPackage (
        wasiArgs
        // {
          cargoArtifacts = wasiArtifacts;
          installPhaseCommand = ''
            mkdir -p $out/bin
            cp target/wasm32-wasip1/release/rnadraw.wasm $out/bin/
          '';
        }
      );
    in
    {
      packages = {
        inherit cli wasm wasi;
        default = cli;
      };

      devShells.default = craneLib.devShell {
        packages = with pkgs; [
          wasm-bindgen-cli
          wasmtime
        ];
      };
    };
}

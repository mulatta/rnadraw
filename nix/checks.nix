{ self, ... }:
{
  perSystem =
    {
      craneLib,
      pkgs,
      config,
      ...
    }:
    let
      src = pkgs.lib.cleanSourceWith {
        src = self;
        filter =
          path: type: (craneLib.filterCargoSources path type) || (builtins.match ".*\\.json$" path != null);
      };

      commonArgs = {
        inherit src;
        strictDeps = true;
      };

      cargoArtifacts = craneLib.buildDepsOnly commonArgs;
    in
    {
      checks = {
        clippy = craneLib.cargoClippy (
          commonArgs
          // {
            inherit cargoArtifacts;
            cargoClippyExtraArgs = "--workspace -- -D warnings";
          }
        );

        test = craneLib.cargoTest (
          commonArgs
          // {
            inherit cargoArtifacts;
            cargoTestExtraArgs = "--workspace";
          }
        );

        inherit (config.packages) cli wasm wasi;
      };
    };
}

_: {
  perSystem =
    { craneLib, pkgs, ... }:
    {
      devShells.default = craneLib.devShell {
        packages = with pkgs; [
          wasm-bindgen-cli
          wasmtime
        ];
      };
    };
}

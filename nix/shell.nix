{
  formatter,
  pkgs,
  wasm-bindgen-cli,
}:

{
  default = pkgs.mkShell {
    packages = [
      pkgs.cargo
      pkgs.clippy
      pkgs.rustc
      pkgs.rustfmt
      wasm-bindgen-cli
      pkgs.wasmtime
      formatter
    ];
  };
}

{
  formatter,
  pkgs,
}:

{
  default = pkgs.mkShell {
    packages = with pkgs; [
      cargo
      clippy
      rustc
      rustfmt
      wasm-bindgen-cli
      wasmtime
      formatter
    ];
  };
}

{ packages }:

{
  package-rnadraw = packages.cli;
  package-rnadraw-wasi = packages.wasi;
  package-rnadraw-wasm = packages.wasm;
  formatting = packages.formatter.passthru.tests.check;
}

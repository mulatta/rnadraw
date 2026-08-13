{
  rustPlatform,
  source,
}:

rustPlatform.buildRustPackage {
  pname = "rnadraw-wasi";
  version = "0.1.0";

  inherit (source) src;
  cargoLock.lockFile = ../../../Cargo.lock;
  cargoBuildFlags = [
    "-p"
    "rnadraw"
  ];
  CARGO_BUILD_TARGET = "wasm32-wasip1";

  strictDeps = true;
  doCheck = false;
  installPhase = ''
    runHook preInstall
    mkdir -p $out/bin
    cp target/wasm32-wasip1/release/rnadraw.wasm $out/bin/
    runHook postInstall
  '';
}

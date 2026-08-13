{
  rustPlatform,
  source,
}:

rustPlatform.buildRustPackage {
  pname = "rnadraw-wasm-raw";
  version = "0.1.0";

  inherit (source) src;
  cargoLock.lockFile = ../../../Cargo.lock;
  cargoBuildFlags = [
    "-p"
    "rnadraw-wasm"
  ];
  CARGO_BUILD_TARGET = "wasm32-unknown-unknown";

  strictDeps = true;
  doCheck = false;
  installPhase = ''
    runHook preInstall
    mkdir -p $out/lib
    cp target/wasm32-unknown-unknown/release/rnadraw_wasm.wasm $out/lib/
    runHook postInstall
  '';
}

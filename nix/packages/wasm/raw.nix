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
    "--target"
    "wasm32-unknown-unknown"
    "-p"
    "rnadraw-wasm"
  ];

  strictDeps = true;
  doCheck = false;
  installPhase = ''
    runHook preInstall
    mkdir -p $out/lib
    cp target/wasm32-unknown-unknown/release/rnadraw_wasm.wasm $out/lib/
    runHook postInstall
  '';
}

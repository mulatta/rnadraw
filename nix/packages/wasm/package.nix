{
  stdenvNoCC,
  wasm-bindgen-cli,
  rnadraw-wasm-raw,
}:

stdenvNoCC.mkDerivation {
  pname = "rnadraw-wasm";
  version = "0.1.0";

  nativeBuildInputs = [ wasm-bindgen-cli ];

  dontUnpack = true;
  buildPhase = ''
    runHook preBuild
    wasm-bindgen \
      --target web \
      --out-dir $out \
      ${rnadraw-wasm-raw}/lib/rnadraw_wasm.wasm
    runHook postBuild
  '';
  dontInstall = true;
}

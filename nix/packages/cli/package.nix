{
  rustPlatform,
  source,
}:

rustPlatform.buildRustPackage {
  pname = "rnadraw";
  version = "0.1.0";

  inherit (source) src;
  cargoLock.lockFile = ../../../Cargo.lock;
  cargoBuildFlags = [
    "-p"
    "rnadraw"
  ];

  strictDeps = true;
  doCheck = false;
}

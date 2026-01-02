{
  lib,
  rustPlatform,
}:

rustPlatform.buildRustPackage {
  pname = "muxwarden";
  version = "1.0.0";

  src = ../../..;

  cargoHash = "sha256-2r+WSykau+5Rvh8cWmgkInyddjh2bL6myEQ16Va8jZY=";

  meta = {
    description = "A TUI for managing SSH multiplex connections and port forwarding";
    homepage = "https://github.com/breakds/muxwarden";
    license = lib.licenses.mit;
    mainProgram = "muxwarden";
    platforms = lib.platforms.linux;
  };
}

{
  lib,
  rustPlatform,
}:

rustPlatform.buildRustPackage {
  pname = "muxwarden";
  version = "1.0.0";

  src = ../../..;

  cargoHash = "sha256-6JP344hliNkGHVqC4OP7H7ZLTw/XyCV2+ffA1w4MV2M=";

  meta = {
    description = "A TUI for managing SSH multiplex connections and port forwarding";
    homepage = "https://github.com/breakds/muxwarden";
    license = lib.licenses.mit;
    mainProgram = "muxwarden";
    platforms = lib.platforms.linux;
  };
}

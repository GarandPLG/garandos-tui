{
  lib,
  rustPlatform,
  pkg-config,
}:
rustPlatform.buildRustPackage {
  name = "garandos-tui";
  src = ./.;
  # buildInputs = [ ];
  nativeBuildInputs = [pkg-config];
  cargoHash = lib.fakeHash;
}

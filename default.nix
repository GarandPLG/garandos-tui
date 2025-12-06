{
  rustPlatform,
  pkg-config,
}:
rustPlatform.buildRustPackage {
  name = "garandos-tui";
  src = ./.;
  # buildInputs = [ ];
  nativeBuildInputs = [pkg-config];
  cargoHash = "sha256-cFAkKwgLzj6Hr2pq7W/1Ps1G3yKzgEam/qV6p31gadA=";
}

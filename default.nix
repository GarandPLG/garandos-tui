{
  lib,
  rustPlatform,
  pkg-config,
}:
rustPlatform.buildRustPackage {
  name = "garandos-tui";
  pname = "garandos-tui";
  version = "0.1.0";

  src = ./.;
  # buildInputs = [ ];
  nativeBuildInputs = [pkg-config];
  cargoHash = "sha256-cFAkKwgLzj6Hr2pq7W/1Ps1G3yKzgEam/qV6p31gadA=";

  meta = {
    description = "TUI for managing GarandsOS' hosts enabled modules";
    homepage = "https://gitea.garandplg.com/GarandPLG/garandos-tui";
    license = lib.licenses.mit;
    maintainers = [
      "Garand_PLG"
    ];
  };
}

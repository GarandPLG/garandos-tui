{
  description = "TUI for managing GarandsOS' hosts enabled modules";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs?ref=nixos-unstable";

    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
    };

    naersk = {
      url = "github:nix-community/naersk";
      inputs = {
        nixpkgs.follows = "nixpkgs";
        fenix.follows = "fenix";
      };
    };
  };

  outputs = {
    self,
    nixpkgs,
    naersk,
    fenix,
  }: let
    system = "x86_64-linux";
    pkgs = nixpkgs.legacyPackages.${system};

    rustToolchain = fenix.packages.${system}.stable.toolchain;
    naerskLib = pkgs.callPackage naersk {
      cargo = rustToolchain;
      rustc = rustToolchain;
    };
  in {
    packages.${system} = {
      default =
        pkgs.callPackage ./default.nix {};

      develop = naerskLib.buildPackage {
        name = "garandos-tui";
        src = ./.;
        nativeBuildInputs = with pkgs; [pkg-config];
      };
    };

    nixosModules.garandos-tui = {
      config,
      lib,
      ...
    }: let
      inherit (lib) mkOption mkIf types;
      cfg = config.programs.garandos-tui;
    in {
      options.programs.garandos-tui = {
        enable = mkOption {
          type = types.bool;
          default = false;
          description = "Enable the Garandos TUI module";
        };
        package = mkOption {
          type = types.package;
          default = self.packages.${system}.default;
          defaultText = "self.packages.${system}.default";
          description = "The Garandos TUI package";
        };
        systemModulesFilePath = mkOption {
          type = types.path;
          default = "";
          example = "/home/\${username}/garandos/hosts/\${hostname}/system-modules.nix";
          description = "The path to the Host's system modules file";
        };
        homeModulesFilePath = mkOption {
          type = types.path;
          default = "";
          example = "/home/\${username}/garandos/hosts/\${hostname}/home-modules.nix";
          description = "The path to the Host's home modules file";
        };
      };
      config = mkIf (cfg.enable && cfg.systemModulesFilePath != "" && cfg.homeModulesFilePath != "") {
        environment.systemPackages = [
          (pkgs.writeScriptBin "garandos-tui" ''
            #!${pkgs.runtimeShell}
            exec ${cfg.package}/bin/garandos_tui \
              --sf '${cfg.systemModulesFilePath}' \
              --hf '${cfg.homeModulesFilePath}' \
              "$@"
          '')
        ];
      };
    };

    devShells.${system}.default = pkgs.mkShell {
      env.RUST_SRC_PATH = "${pkgs.rust.packages.stable.rustPlatform.rustLibSrc}";
      buildInputs = [
        rustToolchain
      ];
      nativeBuildInputs = with pkgs; [pkg-config];

      shellHook = ''
        echo "Commands:"
        echo "    nix build            - Build production version"
        echo "    nix run              - Run production version"
        echo "    nix build .#develop  - Build development version"
        echo "    nix run .#develop    - Run development version"
      '';
    };
  };
}

_: {
  /*
  Container & Packaging
  */
  docker.enable = true; # Docker: container runtime and management
  flatpak = {
    enable = true; # Flatpak: universal packaging system for Linux
    packages = {
      sober.enable = false; # Roblox client
      warehouse.enable = true; # Flatpak manager
      flatseal.enable = true; # Flatpak permissions manager
    };
  };

  /*
  Gaming
  */
  gamemode.enable = true; # GameMode: optimizes system performance for gaming
  gamescope.enable = false; # Gamescope: micro‑compositor for games
  steam.enable = true; # Steam: platform for buying and playing games

  packages = {
    /*
    Container & Packaging
    */
    distrobox.enable = false; # Distrobox: containerized development environments
    lazydocker.enable = false; # Lazydocker: simple TUI for Docker

    /*
    Gaming
    */
    prismlauncher.enable = false; # Prism Launcher: Minecraft modded launcher
    spaceCadetPinball.enable = true; # SpaceCadet Pinball: classic pinball game
    ttySolitaire.enable = true; # TTY Solitaire: terminal‑based solitaire game

    /*
    Development Tools
    */
    exercism.enable = true; # Exercism: coding practice platform
    lazygit.enable = false; # Lazygit: simple TUI for Git
    opencode.enable = true; # OpenCode: tools for coding and development
    jan.enable = true; # Jan: AI chat UI

    /*
    Communication & Collaboration
    */
    mattermost.enable = true; # Mattermost: open‑source Slack alternative
    slack.enable = true; # Slack: team communication and collaboration tool
    tutanota.enable = true; # Tutanota: secure email client

    /*
    Productivity / Knowledge Management
    */
    bitwarden.enable = false; # Bitwarden: password manager (desktop)
    iotas.enable = true; # Iotas: lightweight notes manager
    logseq.enable = false; # Logseq: knowledge base and outliner

    /*
    Media & Graphics
    */
    affinity.enable = false; # Affinity: professional graphics suite
    eyeOfGnome.enable = true; # Eye of GNOME: image viewer
    freetube.enable = false; # FreeTube: privacy‑friendly YouTube client
    gimp.enable = false; # GIMP: GNU Image Manipulation Program
    kdenlive.enable = false; # Kdenlive: video editing software
    plex.enable = true; # Plex: media player and server client

    /*
    Utilities / Misc
    */
    eddieAirVPN.enable = true; # Eddie AirVPN: VPN client
    galculator.enable = true; # Galculator: simple calculator
    gedit.enable = false; # Gedit: GNOME text editor
    winboat.enable = false; # Winboat: Windows remote desktop via RDP
  };
}

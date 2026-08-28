{ lib, ... }: {
  imports = [
    ../core/default.nix

    (lib.modules.importApply ../parts/env.nix { moduleSystem = "nixos"; })
    (lib.modules.importApply ../parts/manifest.nix { moduleSystem = "nixos"; })
    (lib.modules.importApply ../parts/neededForUsers.nix { moduleSystem = "nixos"; })
    (lib.modules.importApply ../parts/path.nix { moduleSystem = "nixos"; })
    (lib.modules.importApply ../parts/permissions.nix { moduleSystem = "nixos"; })

    ./activate
  ];
}

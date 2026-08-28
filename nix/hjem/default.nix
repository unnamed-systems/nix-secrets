{ lib, ... }: {
  imports = [
    ../core/default.nix

    (lib.modules.importApply ../parts/env.nix { moduleSystem = "hjem"; })
    (lib.modules.importApply ../parts/manifest.nix { moduleSystem = "hjem"; })
    (lib.modules.importApply ../parts/neededForUsers.nix { moduleSystem = "hjem"; })
    (lib.modules.importApply ../parts/path.nix { moduleSystem = "hjem"; })
    (lib.modules.importApply ../parts/permissions.nix { moduleSystem = "hjem"; })

    ./activate
  ];
}

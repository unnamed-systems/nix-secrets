{ lib, ... }: {
  imports = [
    ../core/default.nix

    (lib.modules.importApply ../parts/env.nix { moduleSystem = "nix-darwin"; })
    (lib.modules.importApply ../parts/manifest.nix { moduleSystem = "nix-darwin"; })
    (lib.modules.importApply ../parts/neededForUsers.nix { moduleSystem = "nix-darwin"; })
    (lib.modules.importApply ../parts/path.nix { moduleSystem = "nix-darwin"; })
    (lib.modules.importApply ../parts/permissions.nix { moduleSystem = "nix-darwin"; })

    ./activate
  ];
}

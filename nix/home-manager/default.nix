{ lib, ... }: {
  imports = [
    ../core/default.nix

    (lib.modules.importApply ../parts/env.nix { moduleSystem = "home-manager"; })
    (lib.modules.importApply ../parts/path.nix { moduleSystem = "home-manager"; })

    ./activate
  ];
}

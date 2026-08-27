{ lib, ... }: {
  imports = [
    ../core/default.nix

    (lib.modules.importApply ../parts/env.nix { moduleSystem = "nixos"; })
    (lib.modules.importApply ../parts/path.nix { moduleSystem = "nixos"; })

    ./activate
  ];
}

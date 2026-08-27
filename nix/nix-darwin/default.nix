{ lib, ... }: {
  imports = [
    ../core/default.nix

    (lib.modules.importApply ../parts/env.nix { moduleSystem = "nix-darwin"; })
    (lib.modules.importApply ../parts/path.nix { moduleSystem = "nix-darwin"; })

    ./activate
  ];
}

{
  description = "TODO";

  inputs.nix-secrets.url = "path:../../.";

  outputs = { self, nix-secrets, ... }: {
    nixosConfigurations = {
      "shared" = nix-secrets.inputs.nixpkgs.lib.nixosSystem {
        modules = [
          nix-secrets.nixosModules.default
          ./default.nix
          {
            nixpkgs.hostPlatform = "x86_64-linux";
            fileSystems."/".device = "/dev/vda";
            fileSystems."/".fsType = "ext4";
            boot.loader.grub.device = "nodev";
          }
        ];
      };
    };
  };
}

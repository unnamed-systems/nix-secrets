{ config, ... }: {
  nixpkgs.hostPlatform = "x86_64-linux";
  system.stateVersion = config.system.nixos.release;
  fileSystems."/" = {
    device = "/dev/vda";
    fsType = "ext4";
  };
  boot.loader.grub.device = "nodev";

  security.nix-secrets = {
    enable = true;
  };
}

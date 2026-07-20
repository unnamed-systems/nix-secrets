{ lib, pkgs, ... }:
{
  imports = [
    ./manifest.nix
    ./secrets.nix
  ];

  options.security.nix-secrets = {
    enable = lib.mkEnableOption "TODO";

    package = lib.mkOption {
      description = "TODO";
      type = lib.types.package;
      default = pkgs.callPackage ../../package/default.nix { };
    };
  };
}

{ lib, config, pkgs, ... }:
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
      default = pkgs.callPackage ../../package.nix { };
    };

    nixEvalCommand = lib.mkOption {
      description = "TODO";
      type = lib.types.string;
      default = "${lib.getExe config.nix.package} --extra-experimental-features \"nix-command flakes\" eval --raw";
    };
  };
}

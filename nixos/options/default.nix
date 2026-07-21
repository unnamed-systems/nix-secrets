{
  lib,
  config,
  pkgs,
  ...
}:
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
      type = lib.types.str;
      default = "${lib.getExe config.nix.package} --extra-experimental-features \"nix-command flakes\" eval --raw";
      defaultText = lib.literalExpression ''''${lib.getExe config.nix.package} --extra-experimental-features "nix-command flakes" eval --raw'';
      # example = TODO;
    };

    recipientAliases = lib.mkOption {
      description = "TODO";
      type = lib.types.attrsOf (
        lib.types.oneOf [
          (lib.types.listOf lib.types.str)
          lib.types.str
        ]
      );
      default = { };
      apply = builtins.mapAttrs (_name: value: lib.toList value);
      # example = TODO;
    };
  };
}

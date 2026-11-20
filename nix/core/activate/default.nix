{
  lib,
  config,
  pkgs,
  ...
}:
let
  cfg = config.security.nix-secrets;
in
{
  options.security.nix-secrets.activate = {
    enable = lib.mkOption {
      description = ''
        Whether to enable automatic activation of nix-secrets secrets and templates.
      '';
      type = lib.types.bool;
      default = true;
      example = false;
    };

    package = lib.mkOption {
      description = ''
        The nix-secrets package used in `security.nix-secrets.activate.command`. This
        is a dedicated activation package, separate from the regular nix-secrets CLI.
      '';
      type = lib.types.package;
      default = pkgs.callPackage ../../../packages/nix-secrets-activate.nix {
        inherit (cfg) extraPackages;
        nix-secrets-activate-unwrapped =
          pkgs.callPackage ../../../packages/nix-secrets-activate-unwrapped.nix
            { debugBuild = cfg.ciMode.enableDangerously && cfg.ciMode.debugPackage; };
      };
      # example = TODO;
    };

    command = lib.mkOption {
      description = ''
        Command used to activate nix-secrets secrets and templates.

        The function receives whether it should activate secrets and templates
        with `neededForUsers` enabled and returns the command to execute.
      '';
      type = lib.types.functionTo lib.types.str;
      default =
        neededForUsers:
        "${lib.getExe cfg.activate.package} ${cfg.manifest { }} --needed-for-users ${lib.boolToString neededForUsers}";
      defaultText = lib.literalExpression ''neededForUsers: "''${lib.getExe cfg.activate.package} ''${cfg.manifest { }} --needed-for-users ''${lib.boolToString neededForUsers}"'';
      # example = TODO;
    };
  };
}

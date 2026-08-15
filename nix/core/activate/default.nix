{
  lib,
  pkgs,
  config,
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

    command = lib.mkOption {
      description = ''
        Command used to activate nix-secrets secrets and templates.

        The function receives whether it should activate secrets and templates
        with `neededForUsers` enabled and returns the command to execute.
      '';
      type = lib.types.functionTo lib.types.str;
      default =
        neededForUsers:
        "${lib.getExe cfg.package} activate ${pkgs.writeText "nix-secrets-manifest.json" cfg.manifest} --needed-for-users ${lib.boolToString neededForUsers}";
      defaultText = lib.literalExpression ''neededForUsers: "''${lib.getExe cfg.package} activate ''${builtins.toJSON "nix-secrets-manifest.json" cfg.manifest} --needed-for-users ''${lib.boolToString neededForUsers}"'';
      # example = TODO;
    };
  };
}

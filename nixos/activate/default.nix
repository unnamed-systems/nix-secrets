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
  imports = [
    ./activationScripts.nix
    ./systemd.nix
  ];

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

    method = lib.mkOption (
      lib.fix (self: {
        description = ''
          Method used to activate nix-secrets secrets and templates.

          The `systemd` method uses systemd services, while the `activationScripts`
          method uses the system activation scripts mechanism.

          The `systemd` method requires `systemd.sysusers.enable` or
          `services.userborn.enable` to be enabled. It does not work correctly for
          secrets and templates with `neededForUsers` enabled without one of these options.
        '';
        type = lib.types.enum [
          "systemd"
          "activationScripts"
        ];
        default =
          if config.systemd.sysusers.enable or false || config.services.userborn.enable or false then
            "systemd"
          else
            "activationScripts";
        defaultText = lib.literalExpression ''
          if config.systemd.sysusers.enable or false || config.services.userborn.enable or false 
          then "systemd"
          else "activationScripts";
        '';
        example =
          {
            systemd = "activationScripts";
            activationScripts = "systemd";
          }
          .${self.default};
      })
    );
  };
}

{
  lib,
  config,
  ...
}:
{
  imports = [
    ./activationScripts.nix
    ./systemd.nix
  ];

  options.security.nix-secrets.activate.method = lib.mkOption (
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
      defaultText = lib.literalExpression (
        lib.removeSuffix "\n" ''
          if config.systemd.sysusers.enable or false || config.services.userborn.enable or false
          then "systemd"
          else "activationScripts"
        ''
      );
      example =
        {
          systemd = "activationScripts";
          activationScripts = "systemd";
        }
        .${self.default};
    })
  );
}

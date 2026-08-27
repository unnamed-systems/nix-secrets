{
  lib,
  pkgs,
  ...
}:
{
  imports = [
    ./launchd.nix
    ./systemd.nix
  ];

  options.security.nix-secrets.activate.method = lib.mkOption (
    lib.fix (self: {
      description = ''
        Method used to activate nix-secrets secrets and templates.

        The `systemd` method uses systemd services, while the `launchd`
        method uses the launchd agents.
      '';
      type = lib.types.enum [
        "systemd"
        "launchd"
      ];
      default = if pkgs.stdenv.hostPlatform.isLinux then "systemd" else "launchd";
      defaultText = lib.literalExpression (
        lib.removeSuffix "\n" ''
          if pkgs.stdenv.hostPlatform.isLinux
          then "systemd"
          else "launchd"
        ''
      );
      example =
        {
          systemd = "launchd";
          launchd = "systemd";
        }
        .${self.default};
    })
  );
}

{
  lib,
  config,
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
    command = lib.mkOption {
      description = "TODO";
      type = lib.types.functionTo lib.types.str;
      default =
        neededForUsers:
        "${lib.getExe cfg.package} activate ${builtins.toFile "nix-secrets-manifest.json" cfg.manifest} --neededForUsers ${lib.boolToString neededForUsers}";
      # example = TODO;
    };

    method = lib.mkOption (
      lib.fix (self: {
        description = "TODO";
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

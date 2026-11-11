{ moduleSystem }:
assert builtins.elem moduleSystem [ "nixos" ];
{
  lib,
  config,
  ...
}:
let
  cfg = config.security.nix-secrets;

  script = ''
    (
      export PATH=${lib.escapeShellArg (lib.makeBinPath cfg.extraPackages)};
      ${cfg.activate.command false} --dry
      ${cfg.activate.command true} --dry
    )
  '';
in
{
  options.security.nix-secrets.checks.activate.enable = lib.mkOption {
    description = "Whether to check `nix-secrets` activation before switching to a new generation.";
    type = lib.types.bool;
    default = true;
  };

  config = lib.mkIf (cfg.enable && cfg.checks.activate.enable) (
    {
      nixos = {
        system.preSwitchChecks.nix-secrets-activate-check = script;
      };
    }
    .${moduleSystem}
  );
}

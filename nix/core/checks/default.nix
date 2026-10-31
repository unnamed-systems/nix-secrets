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
  options.security.nix-secrets.checks = lib.mkOption {
    description = ''
      Nix eval-time checks for nix-secrets secrets and templates.
    '';
    type = lib.types.attrsOf (
      lib.types.submodule (
        { name, ... }: {
          options = {
            name = lib.mkOption {
              default = name;
              type = lib.types.str;
            };
            state = lib.mkOption {
              description = ''
                Whether this check generates warnings, errors, or nothing at all.
              '';
              default = if cfg.ciMode.enableDangerously then "warn" else "error";
              type = lib.types.enum [
                "disabled"
                "warn"
                "error"
              ];
            };
            warnings = lib.mkOption {
              description = ''
                The actual warnings that get added to the top-level `warnings` attr.
              '';
              default = [ ];
              type = lib.types.listOf lib.types.str;
            };
            assertions = lib.mkOption {
              description = ''
                The actual assertions that get added to the top-level `assertions` attr.
              '';
              default = [ ];
              example = [
                {
                  assertion = false;
                  message = "you can't enable this for that reason";
                }
              ];
              type = lib.types.listOf lib.types.unspecified;
            };
          };
        }
      )
    );
  };

  config = {
    warnings = lib.flatten (
      lib.mapAttrsToList (_: c: lib.optionals (c.state == "warn") c.warnings) cfg.checks
    );
    assertions = lib.flatten (
      lib.mapAttrsToList (_: c: lib.optionals (c.state == "error") c.assertions) cfg.checks
    );
  };

  imports = [
    ./missing.nix
  ];
}

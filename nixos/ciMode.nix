{
  lib,
  config,
  ...
}:
let
  cfg = config.security.nix-secrets;
in
{
  options.security.nix-secrets.ciMode = {
    enableDangerously = lib.mkOption {
      description = "TODO";
      type = lib.types.bool;
      default = false;
    };

    storePathIdentities = lib.mkOption {
      description = "TODO";
      type = lib.types.bool;
      default = false;
    };
  };

  config.security.nix-secrets.ciMode.enableDangerously = false;

  config.warnings = lib.optional cfg.ciMode.enableDangerously ''
    security.nix-secrets.ciMode.enableDangerously IS ENABLED.

    This configuration is INSECURE and must NEVER be deployed to a real
    machine or production environment.

    Stop this rebuild immediately unless you are intentionally running in
    a disposable CI or VM environment.
  '';
}

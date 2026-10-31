{
  lib,
  pkgs,
  config,
  ...
}:
let
  cfg = config.security.nix-secrets;
  missing = lib.filterAttrs (
    _: secret: !lib.pathExists (lib.path.append cfg.storage "${secret.name}.enc")
  ) cfg.secrets;
  can_i_has_generator =
    secret:
    lib.optionalString (
      !isNull secret.generator
    ) "(generate with `nix-secrets regenerate '${secret.name}'`)";
  message = secret: "missing secret: ${secret.name} ${can_i_has_generator secret}";
in
{
  config.security.nix-secrets.checks.missing = lib.mkDefault {
    warnings = lib.mapAttrsToList (_: message) missing;
    assertions = lib.mapAttrsToList (_: secret: {
      assertion = false;
      message = message secret;
    }) missing;
  };
}

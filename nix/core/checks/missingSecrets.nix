{
  lib,
  config,
  ...
}:
let
  cfg = config.security.nix-secrets;

  mkCheck =
    secret_attr: secret:
    let
      encPath = cfg.storage + "/${secret.name}.enc";
      generatorHint = lib.optionalString (
        secret.generator != null
      ) " Generate it with `nix-secrets regenerate '${secret.name}'`.";
    in
    {
      assertion = !secret.enable || builtins.pathExists encPath;
      message = "Secret `security.nix-secrets.secrets.${lib.strings.escapeNixIdentifier secret_attr}` is enabled, but its encrypted file `${toString encPath}` does not exist.${generatorHint}";
    };
in
{
  options.security.nix-secrets.checks.missingSecrets.enable = lib.mkOption {
    description = "Whether to check that every enabled secret has a corresponding encrypted file.";
    type = lib.types.bool;
    default = true;
  };

  config.assertions = lib.optionals cfg.checks.missingSecrets.enable (
    lib.mapAttrsToList mkCheck cfg.secrets
  );
}

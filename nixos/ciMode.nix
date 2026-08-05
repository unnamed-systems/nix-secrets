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
      description = ''
        Whether to enable insecure CI mode options.

        This option enables features that may weaken secret protection and should only
        be used in disposable CI environments or isolated test environments.
      '';
      type = lib.types.bool;
      default = false;
    };

    usePlaceholders = lib.mkOption {
      description = ''
        Whether to use placeholder values instead of decrypted secrets.

        This is intended for CI and testing environments where real secret values
        are not available or should not be used.
      '';
      type = lib.types.bool;
      default = false;
    };
    storePathIdentities = lib.mkOption {
      description = ''
        Whether to allow identity files stored in the Nix store.

        Identity files should not normally be stored in the Nix store because its
        contents are world-readable and therefore unsuitable for keeping confidential
        information.
      '';
      type = lib.types.bool;
      default = false;
    };
    debugPackage = lib.mkOption {
      description = ''
        Whether to use a debug build of `security.nix-secrets.package`.

        The debug build enables additional logging.
      '';
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

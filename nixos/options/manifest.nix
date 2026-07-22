{
  lib,
  config,
  ...
}:
let
  cfg = config.security.nix-secrets;
in
{
  options.security.nix-secrets.manifest = lib.mkOption {
    description = "TODO";
    type = lib.types.str;
    default = builtins.toJSON {
      inherit (cfg) secrets;
    };
    readOnly = true;
    internal = true;
  };
}

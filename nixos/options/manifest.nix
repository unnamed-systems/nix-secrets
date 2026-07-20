{
  lib,
  options,
  config,
  ...
}:
let
  opt = options.security.nix-secrets;
  cfg = config.security.nix-secrets;
in
{
  options.security.nix-secrets.manifest = lib.mkOption {
    description = "TODO";
    type = lib.types.submodule {

      options = {
        inherit (opt) secrets;
      };

      config = {
        inherit (cfg) secrets;
      };

    };
    apply = builtins.toJSON;
    default = { };
    readOnly = true;
    internal = true;
  };
}

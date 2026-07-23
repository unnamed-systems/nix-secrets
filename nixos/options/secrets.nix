{ lib, config, ... }:
let
  cfg = config.security.nix-secrets;

  secretSubmodule = lib.types.submodule (
    { name, config, ... }:
    {
      options = {

        name = lib.mkOption {
          description = "TODO";
          type = lib.types.str;
          default = name;
          # example = TODO;
        };

        recipients = lib.mkOption {
          description = "TODO";
          type = lib.types.listOf lib.types.str;
          default = [ ];
          apply =
            values:
            lib.uniqueStrings (builtins.concatMap (value: cfg.recipientAliases.${value} or [ value ]) values);
          # example = TODO;
        };

        placeholder = lib.mkOption {
          description = "TODO";
          type =
            let
              inputType = lib.types.either lib.types.str lib.types.path;
              outputType = lib.types.path;

              transformFunction =
                value:
                if builtins.isPath value then
                  value
                else
                  # Deduplicates placeholder files automatically.
                  builtins.toFile "nix-secrets-placeholder" value;
            in
            lib.types.coercedTo inputType transformFunction outputType;
          default = "REPLACE WITH YOUR SECRET"; # TODO: generator
          # example = TODO;
        };

        neededForUsers = lib.mkOption {
          description = "TODO";
          type = lib.types.bool;
          default = false;
          example = true;
        };

        # Mounting
        path = lib.mkOption {
          description = "TODO";
          type = lib.types.str;
          # Use separate directories because activation scripts with
          # `beforeUsers = true` and `beforeUsers = false` run independently.
          default = "/run/nix-secrets${lib.optionalString config.neededForUsers "-for-users"}/${config.name}";
          # example = TODO;
        };

        mode = lib.mkOption {
          description = "TODO";
          type = lib.types.str;
          default = "0400";
          # example = TODO;
        };

        owner = lib.mkOption {
          description = "TODO";
          type = lib.types.oneOf [
            lib.types.int
            lib.types.str
          ];
          default = 0;
          # example = TODO;
        };

        group = lib.mkOption {
          description = "TODO";
          type = lib.types.oneOf [
            lib.types.int
            lib.types.str
          ];
          default = 0;
          # example = TODO;
        };

      };
    }
  );
in
{
  options.security.nix-secrets.secrets = lib.mkOption {
    description = "TODO";
    type = lib.types.attrsOf secretSubmodule;
    default = { };
    # example = TODO;
  };
}

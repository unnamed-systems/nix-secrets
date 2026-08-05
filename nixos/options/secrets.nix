{ lib, config, ... }:
let
  cfg = config.security.nix-secrets;

  secretSubmodule = lib.types.submodule (
    { name, config, ... }:
    {
      options = {

        name = lib.mkOption {
          description = ''
            Secret name used as its path in the storage.

            For example, the secret named `"forgejo/token"` is stored as
            `forgejo/token.enc` in the storage.
          '';
          type = lib.types.str;
          default = name;
          example = "forgejo/token";
        };

        recipients = lib.mkOption {
          description = ''
            Recipients allowed to decrypt the secret.

            Entries may be age recipients, SSH public keys, or aliases defined in
            `security.nix-secrets.recipientAliases`.
          '';
          type = lib.types.listOf lib.types.str;
          default = [ ];
          apply =
            values:
            lib.uniqueStrings (builtins.concatMap (value: cfg.recipientAliases.${value} or [ value ]) values);
          example = [
            "alias"
            "age1ls5m8ml8cdu202xakl56lspqrccgln4kfx8q7c6v7qdex92xryhs03v6re"
            "ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAIEuUsB0HH//1qkvgQMWTEoNd0riZpk+8A5w1Ep2vGKk0"
          ];
        };

        placeholder = lib.mkOption {
          description = ''
            Placeholder content used instead of decrypting secret.

            This option exists for CI and testing purposes. It replaces the decrypted
            secret only when `security.nix-secrets.ciMode.usePlaceholders` is enabled.
            It must not be used for real secrets.
          '';
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
          example = "insecurePassword";
        };

        generator = lib.mkOption {
          description = ''
            Package providing the executable used to generate the secret.
          '';
          type = lib.types.nullOr lib.types.package;
          default = null;
          apply = value: {
            derivation = value.drvPath or value;
            # `lib.getExe` doesn't work here because the generator may be a standalone executable rather than a package containing a `bin` directory.
            executable = if value ? meta.mainProgram then lib.getExe' value value.meta.mainProgram else value;
          };
          # example = TODO;
        };

        neededForUsers = lib.mkOption {
          description = ''
            Whether the secret must be available before users and groups are created.

            Enable this for secrets referenced before or during user creation, such as
            `users.users.<name>.hashedPasswordFile`.
          '';
          type = lib.types.bool;
          default = false;
          example = true;
        };

        # Mounting
        path = lib.mkOption {
          description = ''
            Path where the decrypted secret will be mounted.

            By default, the secret is mounted under "/run/nix-secrets", or under
            "/run/nix-secrets-for-users" when `neededForUsers` is enabled.
          '';
          type = lib.types.str;
          # Use separate directories because activation scripts with
          # `beforeUsers = true` and `beforeUsers = false` run independently.
          default = "/run/nix-secrets${lib.optionalString config.neededForUsers "-for-users"}/secrets/${config.name}";
          example = "/var/lib/forgejo/token";
        };

        mode = lib.mkOption {
          description = ''
            Permissions of the file created at `path`.
          '';
          type = lib.types.str;
          default = "0400";
          example = "0660";
        };

        owner = lib.mkOption {
          description = ''
            Owner of the file created at `path`.

            May be specified as either a user name or a numeric UID.

            Cannot be set when `neededForUsers` is enabled because users and groups are
            not available at this stage of activation. In this case, the file owner is
            set to `root`.
          '';
          type = lib.types.oneOf [
            lib.types.int
            lib.types.str
          ];
          default = 0;
          example = "forgejo";
        };

        group = lib.mkOption {
          description = ''
            Group of the file created at `path`.

            May be specified as either a group name or a numeric GID.

            Cannot be set when `neededForUsers` is enabled because users and groups are
            not available at this stage of activation. In this case, the file group is
            set to `root`.
          '';
          type = lib.types.oneOf [
            lib.types.int
            lib.types.str
          ];
          default = 0;
          example = "users";
        };

        # Templating
        templateKey = lib.mkOption {
          description = ''
            Placeholder string used to reference this secret in templates.
          '';
          type = lib.types.str;
          readOnly = true;
          default = "{{NIX_SECRETS-${builtins.hashString "sha256" config.name}}}";
          defaultText = lib.literalExpression "{{NIX_SECRETS-\${builtins.hashString \"sha256\" config.name}}}";
          # SHA-256 hash of the string "secret"
          example = "{{NIX_SECRETS-2bb80d537b1da3e38bd30361aa855686bde0eacd7162fef6a25fe97bf527a25b}}";
        };

        __toString = lib.mkOption {
          description = ''
            Internal option allowing the secret to be converted to its `templateKey` using `"''${secrets.<name>}"`.
          '';
          type = lib.types.functionTo lib.types.str;
          readOnly = true;
          internal = true;
          default = self: self.templateKey;
          defaultText = lib.literalExpression "self: self.templateKey";
          # SHA-256 hash of the string "secret"
          example = "{{NIX_SECRETS-2bb80d537b1da3e38bd30361aa855686bde0eacd7162fef6a25fe97bf527a25b}}";
        };

      };

      config = {

        recipients = cfg.defaultRecipients;

      };
    }
  );
in
{
  options.security.nix-secrets.secrets = lib.mkOption {
    description = ''
      Secrets managed by nix-secrets.
    '';
    type = lib.types.attrsOf secretSubmodule;
    default = { };
    # example = TODO;
  };
}

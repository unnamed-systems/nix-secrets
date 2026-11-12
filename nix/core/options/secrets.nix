{
  lib,
  config,
  pkgs,
  ...
}:
let
  cfg = config.security.nix-secrets;

  secretSubmodule = lib.types.submodule (
    { name, config, ... }:
    {
      options = {
        enable = lib.mkOption {
          description = ''
            Whether to enable this secret.

            When this option is set to `false`, the secret is not included
            in the resulting manifest, like it does not exist in the first place.
          '';
          type = lib.types.bool;
          default = true;
          example = false;
        };

        name = lib.mkOption {
          description = ''
            Secret name used as its path in the storage.

            For example, the secret named `"forgejo/token"` is stored as
            `forgejo/token.enc` in the storage.
          '';
          type = lib.types.addCheck (lib.types.pathWith {
            absolute = false;
            inStore = false;
          }) (value: builtins.isString value && "/" + value == toString (/. + value));
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

        value =
          let
            host = config.networking.hostName or "idk";
            home = "/tmp";

            # This uses IFD which sucks. But how else could a decrypted secret be evaluated in the config?
            decrypted =
              # The ability to directly pass the manifest to nix-secrets could be nice here
              pkgs.runCommand "nix-secrets-vars-${name}" { nativeBuildInputs = [ cfg.package ]; } ''
                export HOME=${home} \
                  HOST=${host} \
                  NIX_SECRETS_NIX_BUILD_COMMAND='echo "${builtins.toJSON (cfg.manifest { })}"' \
                  NIX_SECRETS_STORAGE_PATH=${cfg.storage}
                nix-secrets decrypt ${config.name}
              '';
          in
          lib.mkOption {
            description = ''
              This variable's value. CAN and WILL end up in the nix store.
            '';
            type = lib.types.str;
            readOnly = true;
            default =
              assert config.isVar;
              if cfg.ciMode.usePlaceholders then config.placeholder else (builtins.readFile decrypted);
            defaultText = lib.literalExpression "if cfg.ciMode.usePlaceholders then self.placeholder else (builtins.readFile (decrypted self))";
            example = "somevariable";
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
              inputType = lib.types.oneOf [
                lib.types.str
                lib.types.path
                lib.types.package
              ];
              outputType = lib.types.path;

              transformFunction =
                value:
                if lib.types.package.check value then
                  value
                else if lib.types.path.check value then
                  value
                else if builtins.getContext value == { } then
                  builtins.toFile "nix-secrets-placeholder" value
                else
                  pkgs.writeText "nix-secrets-placeholder" value;
            in
            lib.types.coercedTo inputType transformFunction outputType;
          default = "REPLACE WITH YOUR SECRET"; # TODO: generator
          example = "insecurePassword";
        };

        generator = lib.mkOption {
          description = ''
            Package used to generate the secret.

            Can be either:
            - a derivation,
            - a generator name from `config.security.nix-secrets.generators`,
            - an attribute set with a single attribute, where the attribute name selects
              a generator from `config.security.nix-secrets.generators` and its value is
              passed to that generator.
          '';
          type = lib.types.nullOr (
            lib.types.oneOf [
              lib.types.package
              lib.types.str
              lib.types.attrs
            ]
          );
          apply =
            let
              isGenerator = value: builtins.isAttrs value && builtins.length (builtins.attrNames value) == 1;

              isGeneratorName = value: builtins.isString value;

              func =
                value:
                if lib.types.package.check value then
                  value
                else if isGenerator value then
                  let
                    name = builtins.head (builtins.attrNames value);
                  in
                  func (cfg.generators.${name} value.${name})
                else if isGeneratorName value then
                  func (cfg.generators.${toString value} { })
                else
                  abort "expected a derivation, generator name, or generator attribute set.";
            in
            lib.mapNullable func;
          default = null;
          # example = TODO;
        };

        # Templating
        templateKey = lib.mkOption {
          description = ''
            Placeholder string used to reference this secret in templates.
          '';
          type = lib.types.str;
          readOnly = true;
          default =
            if config.isVar then
              config.value
            else
              "{{NIX_SECRETS-${builtins.hashString "sha256" config.name}}}";
          defaultText = lib.literalExpression ''"{{NIX_SECRETS-''${builtins.hashString "sha256" config.name}}}"'';
          # SHA-256 hash of the string "secret"
          example = "{{NIX_SECRETS-2bb80d537b1da3e38bd30361aa855686bde0eacd7162fef6a25fe97bf527a25b}}";
        };

        __toString = lib.mkOption {
          description = ''
            Internal option allowing the secret to be converted to its `templateKey` using `"''${secrets.<name>}"`.
          '';
          type = lib.types.functionTo lib.types.str;
          readOnly = true;
          default = self: self.templateKey;
          defaultText = lib.literalExpression "self: self.templateKey";
          # SHA-256 hash of the string "secret"
          example = "{{NIX_SECRETS-2bb80d537b1da3e38bd30361aa855686bde0eacd7162fef6a25fe97bf527a25b}}";
        };

        isVar = lib.mkOption {
          description = ''
            Whether to add this secret (and dedicated identity) to the secrets storage
            location and therefore make it world-readable as plain text in the nix store.

            Secrets marked as variables may be accessed through their `value` attribute.

            Do not set this option unless you know exactly what you are doing. This option
            is mainly intended for values that must be available in the nix store without
            them being committed in plain text to their public git repositories.
          '';
          type = lib.types.bool;
          default = false;
          example = true;
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

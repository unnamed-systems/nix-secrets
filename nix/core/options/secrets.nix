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
            Package used to generate the secret.

            Can be either:
            - a derivation,
            - a generator name from `config.security.nix-secrets.generators`,
            - an attribute set with a single attribute, where the attribute name selects
              a generator from `config.security.nix-secrets.generators` and its value is
              passed to that generator,
            - an attribute set containing `derivation` and optionally `executable`.
              `derivation` can be either a derivation or a path to a `.drv` file.
              `executable` can be either a derivation or a path to an executable file.
          '';
          type = lib.types.nullOr (
            lib.types.oneOf [
              lib.types.attrs
              lib.types.path
              lib.types.str
            ]
          );
          apply =
            let
              isRaw =
                value:
                builtins.isAttrs value
                && builtins.elem (builtins.attrNames value) [
                  [ "derivation" ]
                  [
                    "derivation"
                    "executable"
                  ]
                ];

              isGenerator = value: builtins.isAttrs value && builtins.length (builtins.attrNames value) == 1;

              isGeneratorName = value: builtins.isString value || value ? outPath || value ? __toString;

              func =
                value:
                if isRaw value then
                  {
                    derivation =
                      let
                        v = value.derivation;
                      in
                      v.drvPath or (toString v);

                    executable =
                      let
                        v =
                          value.executable or (
                            if lib.isDerivation value.derivation then
                              value.derivation
                            else
                              abort "`executable` is required when `derivation` is a path rather than a derivation."
                          );
                      in
                      if v ? meta.mainProgram then lib.getExe' v v.meta.mainProgram else toString v;
                  }
                else if lib.isDerivation value then
                  func { derivation = value; }
                else if isGenerator value then
                  let
                    name = builtins.head (builtins.attrNames value);
                  in
                  func (cfg.generators.${name} value.${name})
                else if isGeneratorName value then
                  func (cfg.generators.${toString value} { })
                else
                  abort "expected a derivation, generator name, generator attribute set, or raw generator specification.";
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
          default = "{{NIX_SECRETS-${builtins.hashString "sha256" config.name}}}";
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

{
  lib,
  config,
  pkgs,
  ...
}:
let
  cfg = config.security.nix-secrets;
in
{
  imports = [
    ./manifest.nix
    ./secrets.nix
    ./templates.nix
  ];

  options.security.nix-secrets = {
    enable = lib.mkOption {
      description = "Whether to enable nix-secrets integration.";
      type = lib.types.bool;
      default = false;
      example = true;
    };

    storage = lib.mkOption {
      description = ''
        Path to the nix-secrets storage.

        The path is used by activation scripts and services and must be specified as a path
        rather than a relative path string. For example, use `./storage` instead
        of `"./storage"`.

        For improved security, it is recommended to keep the storage in a private
        repository and reference it through a flake input.
      '';
      type = lib.types.path;
      example = lib.literalExpression "inputs.my-secrets + /storage";
    };

    identityPaths = lib.mkOption {
      description = ''
        Paths to identity files used for secret encryption and decryption.

        For security reasons, identity files are kept outside of the Nix store.
      '';
      type = lib.types.listOf (
        lib.types.pathWith {
          inStore = if cfg.ciMode.enableDangerously && cfg.ciMode.storePathIdentities then null else false;
          absolute = null;
        }
      );
      default = [ ];
      example = [
        "/home/user/keys.txt"
        "/home/user/.ssh/id_ed25519"
      ];
    };

    package = lib.mkOption {
      description = ''
        nix-secrets package used by activation scripts and services.

        It can also be installed into the system by enabling
        `security.nix-secrets.installPackage`, which is enabled by default.
      '';
      type = lib.types.package;
      default = pkgs.callPackage ../../package.nix {
        debugBuild = cfg.ciMode.enableDangerously && cfg.ciMode.debugPackage;
      };
      # example = TODO;
    };

    extraPackages = lib.mkOption {
      description = ''
        Additional packages made available to nix-secrets during activation scripts and services.

        This is primarily intended for age plugins such as `pkgs.age-plugin-yubikey`.
      '';
      type = lib.types.listOf lib.types.package;
      default = [ ];
      example = lib.literalExpression "[ pkgs.age-plugin-yubikey ]";
    };

    recipientAliases = lib.mkOption {
      description = ''
        Recipient aliases used when defining secret recipients.

        An alias may refer to one or more recipients or to other aliases. Alias references
        are resolved recursively. Cyclic references result in an evaluation error.
      '';
      type =
        let
          inputType = lib.types.either lib.types.str (lib.types.listOf lib.types.str);
          outputType = lib.types.listOf lib.types.str;

          transformFunction = lib.toList;
        in
        lib.types.attrsOf (lib.types.coercedTo inputType transformFunction outputType);
      default = { };
      apply =
        aliases:
        let
          func =
            visited:
            builtins.concatMap (
              value:
              if visited ? ${value} then
                throw "Cyclic recipient alias: ${value}"
              else if aliases ? ${value} then
                func (visited // { ${value} = true; }) aliases.${value}
              else
                [ value ]
            );
        in
        builtins.mapAttrs (name: value: lib.uniqueStrings (func { ${name} = true; } value)) aliases;
      example = {
        pc = "age14e2jdmau7tpau9emcn6gmg26vfl0uyf6cfd9lz85jml6ttv9wq2qphps4t";
        server = [
          "age1ls5m8ml8cdu202xakl56lspqrccgln4kfx8q7c6v7qdex92xryhs03v6re"
          "ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAIEuUsB0HH//1qkvgQMWTEoNd0riZpk+8A5w1Ep2vGKk0"
        ];

        all = [
          "pc"
          "server"
        ];
      };
    };
  };
}

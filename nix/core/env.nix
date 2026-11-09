{ lib, config, ... }:
{
  options.security.nix-secrets = {
    nixBuildCommand = lib.mkOption {
      description = ''
        Command used by the nix-secrets CLI to build the manifest for the
        `security.nix-secrets.manifest` option.

        The placeholders listed below are replaced by the CLI and escaped as shell arguments:
        - `{{flakePath}}` - absolute path to the flake passed into `--flake`,
        - `{{outputName}}` - name of the output, e.g. `"nixosConfigurations"`,
        - `{{configurationName}}` - name of the configuration in `outputName` passed after `"#"` in the `--flake` argument,
        - `{{manifestPath}}` - path to the `manifest` option in `config`, e.g. `"security.nix-secrets.manifest"` or `home-manager.users.<--home-manager>.security.nix-secrets.manifest`,
        - `{{manifestArgs}}` - JSON-string with arguments for generating the manifest.
      '';
      type = lib.types.nullOr lib.types.str;
      default = "${
        lib.optionalString (config.nix.package or null != null) "${config.nix.package}/bin/"
      }nix-build --extra-experimental-features 'flakes' --no-out-link --expr '{flakePath, outputName, configurationName, manifestArgs}: (builtins.getFlake flakePath).\${outputName}.\${configurationName}.config.{{manifestPath}} (builtins.fromJSON manifestArgs)' --argstr flakePath {{flakePath}} --argstr outputName {{outputName}} --argstr configurationName {{configurationName}} --argstr manifestArgs {{manifestArgs}}";
      defaultText = lib.literalExpression ''"''${lib.optionalString (config.nix.package or null != null) "''${config.nix.package}/bin/"}/bin/nix-build --extra-experimental-features 'flakes' --no-out-link --expr '{flakePath, outputName, configurationName, manifestArgs}: (builtins.getFlake flakePath).\''${outputName}.\''${configurationName}.config.{{manifestPath}} (builtins.fromJSON manifestArgs)' --argstr flakePath {{flakePath}} --argstr outputName {{outputName}} --argstr configurationName {{configurationName}} --argstr manifestArgs {{manifestArgs}}"'';
      # example = TODO;
    };

    storagePath = lib.mkOption {
      description = ''
        Default storage path used by the nix-secrets CLI.

        The path must be absolute and outside of the Nix store.

        When set, the CLI does not require the `--storage` argument.
      '';
      type = lib.types.nullOr (
        lib.types.pathWith {
          inStore = false;
          absolute = true;
        }
      );
      default = null;
      example = "/etc/nixos/secrets";
    };

    installPackage = lib.mkOption {
      description = "Whether to enable installation of `security.nix-secrets.package` into `environment.systemPackages`.";
      type = lib.types.bool;
      default = true;
      example = false;
    };
  };
}

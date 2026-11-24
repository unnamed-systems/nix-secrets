{ lib, config, ... }: {
  options.security.nix-secrets = {
    nixBuildCommand = lib.mkOption {
      description = ''
        Command used by the nix-secrets CLI to build the manifest for the
        `security.nix-secrets.manifest` option.

        The placeholders listed below are replaced by the CLI and escaped as shell arguments:
        - `{{isFlake}}` - `true` if `--flake` was passed, `false` otherwise,
        - `{{path}}` - absolute path to the flake (for `--flake`) or to the configuration file (for `--file`),
        - `{{moduleSystem}}` - `"nixos"`, `"home-manager"` or `"nix-darwin"`; empty when `{{outputName}}` is set,
        - `{{manifestPath}}` - path to the `manifest` option in `config`, e.g. `"security.nix-secrets.manifest"` or `home-manager.users.<--home-manager>.security.nix-secrets.manifest`,
        - `{{manifestArgs}}` - JSON-string with arguments for generating the manifest,
        - `{{attr}}` - attribute in `{{path}}` to evaluate (for `--file` only, empty for `--flake`),
        - `{{outputName}}` - name of the flake output, e.g. `"nixosConfigurations"` (for `--flake` only, empty for `--file`),
        - `{{configurationName}}` - name of the configuration in `{{outputName}}`, i.e. the part after `"#"` in `--flake` (for `--flake` only, empty for `--file`).
      '';
      type = lib.types.nullOr lib.types.str;
      default = "${
        lib.optionalString (config.nix.package or null != null) "${config.nix.package}/bin/"
      }nix-build ${../../manifest.nix} --extra-experimental-features flakes --no-out-link --arg isFlake {{isFlake}} --argstr path {{path}} --argstr moduleSystem {{moduleSystem}} --argstr manifestPath {{manifestPath}} --argstr manifestArgs {{manifestArgs}} --argstr attr {{attr}} --argstr outputName {{outputName}} --argstr configurationName {{configurationName}}";
      defaultText = lib.literalExpression ''
        "''${
          lib.optionalString (config.nix.package or null != null) "''${config.nix.package}/bin/"
        }nix-build ''${../../manifest.nix} --extra-experimental-features flakes --no-out-link --arg isFlake {{isFlake}} --argstr path {{path}} --argstr moduleSystem {{moduleSystem}} --argstr manifestPath {{manifestPath}} --argstr manifestArgs {{manifestArgs}} --argstr attr {{attr}} --argstr outputName {{outputName}} --argstr configurationName {{configurationName}}"'';
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

{
  lib,
  config,
  pkgs,
  ...
}:
{
  imports = [
    ./manifest.nix
    ./secrets.nix
  ];

  options.security.nix-secrets = {
    enable = lib.mkEnableOption "TODO";

    storage = lib.mkOption {
      description = "TODO";
      type = lib.types.path;
      # example = TODO;
    };

    identityPaths = lib.mkOption {
      description = "TODO";
      type = lib.types.listOf (
        lib.types.pathWith {
          inStore = false;
          absolute = null;
        }
      );
      default = [ ];
      # example = TODO;
    };

    package = lib.mkOption {
      description = "TODO";
      type = lib.types.package;
      default = pkgs.callPackage ../../package.nix { };
    };

    nixEvalCommand = lib.mkOption {
      description = "TODO";
      type = lib.types.str;
      default = "${lib.getExe config.nix.package} --extra-experimental-features \"nix-command flakes\" eval --raw";
      defaultText = lib.literalExpression ''''${lib.getExe config.nix.package} --extra-experimental-features "nix-command flakes" eval --raw'';
      # example = TODO;
    };

    recipientAliases = lib.mkOption {
      description = "TODO";
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
      # example = TODO;
    };
  };
}

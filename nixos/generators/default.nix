{
  lib,
  ...
}:
{
  options.security.nix-secrets.generators = lib.mkOption {
    description = ''
      Predefined secret generators referenced by name from
      `security.nix-secrets.secrets.<name>.generator`.

      Each generator is a function that accepts an attribute set of arguments and
      returns a package or a generator specification.

      A generator without arguments can be referenced by name:

      ```nix
      generator = "uuid";
      ```

      A generator with arguments can be referenced using an attribute set:

      ```
      generator.uuid = {
        count = 5;
        raw = true;
      };
      ```
    '';
    type = lib.types.attrsOf (
      lib.types.functionTo (lib.types.either lib.types.package lib.types.attrs)
    );
    default = { };
    example = lib.literalExpression ''
      {
        password =
          { length ? 32 }:
          pkgs.writeShellScriptBin "generate-password" '''
            head -c ''${toString length} /dev/urandom | base64
          ''';

        uuid = {}: pkgs.writeShellScriptBin "generate-uuid" '''
          uuidgen
        ''';
      }
    '';
  };

  imports = [
    ./default/uuid.nix
  ];
}

{ moduleSystem }:
{ lib, ... }:
let
  descriptions = {
    secrets = ''
      Whether the secret must be available before users and groups are created.

      Enable this for secrets referenced before or during user creation, such as
      `users.users.<name>.hashedPasswordFile`.

      \> [!NOTE]
      \> Only has an effect in NixOS. Forced to `false` in other module systems.
    '';
    templates = ''
      Whether the template must be available before users and groups are created.

      Enable this for templates referenced before or during user creation.

      \> [!NOTE]
      \> Only has an effect in NixOS. Forced to `false` in other module systems.
    '';
  };

  mkNeededForUsers =
    attr:
    lib.mkOption {
      type = lib.types.attrsOf (
        lib.types.submodule {
          options.neededForUsers = lib.mkOption (
            {
              description = descriptions.${attr};
              type = lib.types.bool;
              default = false;
              example = true;
            }
            // lib.optionalAttrs (moduleSystem != "nixos") {
              apply = _: false;
            }
          );
        }
      );
    };
in
{
  options.security.nix-secrets = lib.genAttrs [ "secrets" "templates" ] mkNeededForUsers;
}

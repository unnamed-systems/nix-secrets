{ lib, ... }:
let
  mkDescription =
    attr: option:
    let
      action = if option == "reloadUnits" then "reload" else "restart";
      subject = if attr == "secrets" then "secret" else "template";
      content = if attr == "secrets" then "decrypted content" else "rendered content";
    in
    ''
      Systemd units to ${action} when the deployed ${subject} changes.

      The units are triggered when nix-secrets deploys the ${subject} to a path that
      either does not exist yet or whose existing file does not match its ${content},
      permissions, ownership, or `isSymlink`.

      \> [!NOTE]
      \> On NixOS this triggers system units. On Home Manager and Hjem it triggers
      \> user units. On nix-darwin these options are ignored.
    '';

  mkTrigger =
    attr: option:
    lib.mkOption {
      description = mkDescription attr option;
      type = lib.types.listOf lib.types.str;
      default = [ ];
      example = [ "nginx.service" ];
    };

  mkTriggers =
    attr:
    lib.mkOption {
      type = lib.types.attrsOf (
        lib.types.submodule {
          options = {
            reloadUnits = mkTrigger attr "reloadUnits";
            restartUnits = mkTrigger attr "restartUnits";
          };
        }
      );
    };
in
{
  options.security.nix-secrets = lib.genAttrs [ "secrets" "templates" ] mkTriggers;
}

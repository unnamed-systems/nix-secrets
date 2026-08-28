{
  lib,
  ...
}:
{
  imports = [
    ./systemd.nix
  ];

  options.security.nix-secrets.activate.method = lib.mkOption {
    description = ''
      Method used to activate nix-secrets secrets and templates.
    '';
    type = lib.types.enum [ "systemd" ];
    default = "systemd";
    # example = TODO;
  };
}

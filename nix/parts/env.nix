{ moduleSystem }:
{ lib, config, ... }:
let
  cfg = config.security.nix-secrets;

  packages = lib.optional cfg.installPackage cfg.package;
in
{
  config = lib.mkIf cfg.enable (
    if moduleSystem == "home-manager" then
      {
        home = {
          inherit packages;
        };
      }
    else if moduleSystem == "hjem" then
      {
        inherit packages;
      }
    else
      {
        environment.systemPackages = packages;
      }
  );
}

{ moduleSystem }:
{ lib, config, ... }:
let
  cfg = config.security.nix-secrets;

  variables =
    let
      mkIfNotNull = value: lib.mkIf (value != null) value;
    in
    {
      NIX_SECRETS_NIX_EVAL_COMMAND = mkIfNotNull cfg.nixEvalCommand;
      NIX_SECRETS_GENERATOR_BUILD_COMMAND = mkIfNotNull cfg.generatorBuildCommand;
    };

  packages = lib.optional cfg.installPackage cfg.package;
in
{
  config = lib.mkIf cfg.enable (
    if moduleSystem == "home-manager" then
      {
        home = {
          sessionVariables = variables;
          inherit packages;
        };
      }
    else if moduleSystem == "hjem" then
      {
        inherit packages;
        environment.sessionVariables = variables;
      }
    else
      {
        environment = {
          inherit variables;
          systemPackages = packages;
        };
      }
  );
}

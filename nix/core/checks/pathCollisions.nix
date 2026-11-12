{
  lib,
  config,
  ...
}:
let
  cfg = config.security.nix-secrets;

  addPath =
    kind: acc: name: value:
    let
      path = lib.optional value.enable { inherit kind name; };
      paths = acc.paths.${value.path} or [ ] ++ path;
    in
    {
      found = acc.found || (builtins.length paths > 1);
      paths.${value.path} = paths;
    };

  noCollisions = {
    found = false;
    paths = { };
  };

  secretCollisions = lib.foldlAttrs (addPath "secret") noCollisions cfg.secrets;
  allCollisions = lib.foldlAttrs (addPath "template") secretCollisions cfg.templates;

  mkName = { kind, name }: "${kind} ${lib.strings.escapeNixIdentifier name}";
  mkNames = values: lib.join ", " (lib.map mkName values);

  mkAssertion = path: values: {
    assertion = builtins.length values <= 1;
    message = "Path '${toString path}' in `nix-secrets` is not unique. Used in: ${mkNames values}.";
  };

  assertions =
    if !allCollisions.found then [ ] else (lib.mapAttrsToList mkAssertion allCollisions.paths);
in
{
  options.security.nix-secrets.checks.pathCollisions.enable = lib.mkOption {
    description = "Whether to check that every enabled secret and a template has a unique path.";
    type = lib.types.bool;
    default = true;
    example = false;
  };

  config.assertions = lib.mkIf (cfg.enable && cfg.checks.pathCollisions.enable) assertions;
}

{
  lib,
  makeBinaryWrapper,
  symlinkJoin,
  nix-secrets-unwrapped,
  nixBuildCommand ? null,
}:
let
  inherit (nix-secrets-unwrapped) pname version meta;
in
symlinkJoin {
  inherit pname version meta;

  paths = [ nix-secrets-unwrapped ];

  nativeBuildInputs = [ makeBinaryWrapper ];

  postBuild = lib.optionalString (nixBuildCommand != null) ''
    wrapProgram "$out/bin/${pname}" \
      --set-default NIX_SECRETS_NIX_BUILD_COMMAND ${lib.escapeShellArg nixBuildCommand}
  '';

  passthru.unwrapped = nix-secrets-unwrapped;
}

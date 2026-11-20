{
  lib,
  makeBinaryWrapper,
  symlinkJoin,
  nix-secrets-unwrapped,
  nixBuildCommand ? null,
  # TODO: https://github.com/unnamed-systems/nix-secrets/issues/40
  age,
  extraPackages ? [ age ],
}:
let
  inherit (nix-secrets-unwrapped) pname version meta;
in
symlinkJoin {
  inherit pname version meta;

  paths = [ nix-secrets-unwrapped ];

  nativeBuildInputs = [ makeBinaryWrapper ];

  postBuild = ''
    wrapProgram "$out/bin/${pname}" \
      --prefix PATH : ${lib.makeBinPath extraPackages} \
      ${lib.optionalString (
        nixBuildCommand != null
      ) "--set-default NIX_SECRETS_NIX_BUILD_COMMAND ${lib.escapeShellArg nixBuildCommand}"}
  '';

  passthru.unwrapped = nix-secrets-unwrapped;
}

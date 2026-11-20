{
  lib,
  makeBinaryWrapper,
  symlinkJoin,
  nix-secrets-activate-unwrapped,
  # TODO: https://github.com/unnamed-systems/nix-secrets/issues/40
  age,
  extraPackages ? [ age ],
}:
let
  inherit (nix-secrets-activate-unwrapped) pname version meta;
in
symlinkJoin {
  inherit pname version meta;

  paths = [ nix-secrets-activate-unwrapped ];

  nativeBuildInputs = [ makeBinaryWrapper ];

  postBuild = lib.optionalString (extraPackages != [ ]) ''
    wrapProgram "$out/bin/${pname}" \
      --prefix PATH : ${lib.makeBinPath extraPackages}
  '';

  passthru.unwrapped = nix-secrets-activate-unwrapped;
}

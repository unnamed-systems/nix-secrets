{
  lib,
  makeBinaryWrapper,
  symlinkJoin,
  callPackage,
  unwrapped ? callPackage ./nix-secrets-activate-unwrapped.nix { },
  # TODO: https://github.com/unnamed-systems/nix-secrets/issues/40
  age,
  extraPackages ? [ age ],
}:
let
  wrapperArgs = lib.optionals (extraPackages != [ ]) [
    "--prefix"
    "PATH"
    ":"
    (lib.makeBinPath extraPackages)
  ];
in
symlinkJoin {
  inherit (unwrapped) pname version;
  __structuredAttrs = true;
  strictDeps = true;

  paths = [ unwrapped ];

  nativeBuildInputs = [ makeBinaryWrapper ];

  postBuild = lib.optionalString (wrapperArgs != [ ]) ''
    wrapProgram "$out/bin/${unwrapped.meta.mainProgram}" ${lib.escapeShellArgs wrapperArgs}
  '';

  passthru = {
    inherit unwrapped;
  }
  // unwrapped.passthru;

  meta = {
    # Manually inherit fields to keep `meta.position` pointing to this file.
    inherit (unwrapped.meta)
      description
      homepage
      license
      maintainers
      mainProgram
      platforms
      ;
  };
}

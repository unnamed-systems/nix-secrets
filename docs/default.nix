{
  lib,
  stdenv,
  mdbook,
  nixdoc,
  jq,
  nixosOptionsDoc,
  self,
  callPackage,
  ...
}:
let
  optionsDoc =
    {
      modules ? [ ],
      rev ? "master",
    }:
    let
      eval = lib.evalModules {
        modules = modules ++ [
          {
            _module.check = false;
            _module.args.pkgs.callPackage = callPackage;
          }
        ];
      };

      optionsDoc = nixosOptionsDoc {
        options.security.nix-secrets = eval.options.security.nix-secrets;

        transformOptions =
          opt:
          opt
          // {
            declarations = map (
              decl:
              let
                root = self.outPath;
                declStr = toString (lib.filesystem.resolveDefaultNix decl);
                subpath = lib.removePrefix "/" (lib.removePrefix root declStr);
              in
              assert lib.hasPrefix root declStr;
              {
                url = "https://github.com/unnamed-systems/nix-secrets/blob/${rev}/${toString subpath}";
                name = subpath;
              }
            ) opt.declarations;
          };
      };
    in
    optionsDoc.optionsCommonMark;
in
stdenv.mkDerivation {
  name = "nix-secrets-book";
  version = "0.1.0";

  src = lib.fileset.toSource {
    root = ../.;
    fileset = lib.fileset.unions [
      ./src
      ./book.toml
      ../nix/core/generators/default
    ];
  };

  nativeBuildInputs = [
    mdbook
    nixdoc
    jq
  ];

  patchPhase = ''
    cat ${
      optionsDoc { modules = [ self.nixosModules.default ]; }
    } > docs/src/nixos-configuration-options.md
    cat ${
      optionsDoc { modules = [ self.darwinModules.default ]; }
    } > docs/src/nix-darwin-configuration-options.md

    mkdir -p "''${TMPDIR}"/nixdoc

    for file in ./nix/core/generators/default/*.nix; do
      output="''${TMPDIR}/nixdoc/$(basename "''${file}").md"

      nixdoc \
        --prefix "" \
        --anchor-prefix "generator-" \
        --category "" \
        --description "" \
        --file "''${file}" \
        > "$output"
    done

    echo "# Generators Reference" > ./docs/src/generators_reference.md
    cat "''${TMPDIR}"/nixdoc/*.md >> ./docs/src/generators_reference.md

    sed -i \
        's/config\.security\.nix-secrets\.generators\.//g' \
        ./docs/src/generators_reference.md
  '';

  buildPhase = ''
    cd docs
    mdbook build

    cp -r $TMPDIR/nixdoc book/LOL
  '';

  installPhase = "cp -r book $out";
}

{
  isFlake,
  path,
  moduleSystem,
  manifestPath,
  manifestArgs,
  # isFlake = false
  attr,
  # isFlake = true
  outputName,
  configurationName,
}:
# types
assert (builtins.isBool isFlake) || throw "`isFlake` must be a boolean";
assert
  (builtins.all builtins.isString [
    path
    moduleSystem
    manifestPath
    manifestArgs
    attr
    outputName
    configurationName
  ])
  || throw "All arguments except `isFlake` must be strings";
# shared
assert (path != "") || throw "`path` is required";
assert (manifestPath != "") || throw "`manifestPath` is required";
assert (manifestArgs != "") || throw "`manifestArgs` is required";
assert
  (
    moduleSystem != ""
    -> builtins.elem moduleSystem [
      "nixos"
      "home-manager"
      "nix-darwin"
      # Hjem is not listed because standalone Hjem consumes its manifest file, whereas nix-secrets is a module.
    ]
  )
  || throw "Unknown module system: ${builtins.toJSON moduleSystem}";
# flakes
assert (isFlake -> (attr == "")) || throw "Flakes do not support `attr`";
assert (isFlake -> (configurationName != "")) || throw "Flakes require `configurationName`";
assert
  (isFlake -> ((moduleSystem != "") != (outputName != "")))
  || throw "Flakes require either `moduleSystem` or `outputName`, but not both";
# non-flakes
assert (!isFlake -> (moduleSystem != "")) || throw "Non-flakes require `moduleSystem`";
assert (!isFlake -> (outputName == "")) || throw "Non-flakes do not support `outputName`";
assert
  (!isFlake -> (configurationName == "")) || throw "Non-flakes do not support `configurationName`";
let
  flakeConfiguration =
    (builtins.getFlake path).${
      if outputName != "" then
        outputName
      else
        {
          nixos = "nixosConfigurations";
          home-manager = "homeConfigurations";
          nix-darwin = "darwinConfigurations";
        }
        .${moduleSystem}
    }.${configurationName};

  fileConfiguration =
    let
      configuration = if attr == "" then path else (import path).${attr};
    in
    if moduleSystem == "nixos" then
      import <nixpkgs/nixos> { inherit configuration; }
    # https://github.com/nix-community/home-manager/blob/3b8140d21fa0a70b1b38c2ed29853dfe686234a6/home-manager/home-manager#L234
    else if moduleSystem == "home-manager" then
      import <home-manager/modules> {
        inherit configuration;
        pkgs = import <nixpkgs> { };
        # check = true;
      }
    else if moduleSystem == "nix-darwin" then
      # https://github.com/nix-darwin/nix-darwin/blob/663d8b962a11cda261b6e9f14e03911fac94baf0/pkgs/nix-tools/darwin-rebuild.sh#L195
      import <darwin> { inherit configuration; }
    else
      throw "unreachable";

  inherit (if isFlake then flakeConfiguration else fileConfiguration) config;

  # Does not handle correctly quoted attribute names such as `first."second.still_second".third`.
  getAttrByPath =
    attrPath: default: set:
    let
      parts = builtins.filter (x: builtins.isString x && x != "") (builtins.split "\\." attrPath);
      lenParts = builtins.length parts;
      fn =
        n: s:
        if n == lenParts then
          s
        else
          let
            attr = builtins.elemAt parts n;
          in
          if builtins.isAttrs s && builtins.hasAttr attr s then fn (n + 1) s.${attr} else default;
    in
    fn 0 set;

  manifest = getAttrByPath manifestPath (throw ''
    Manifest is not found at `${manifestPath}`.
    Did you import the nix-secrets module?
  '') config;
in
manifest (builtins.fromJSON manifestArgs)

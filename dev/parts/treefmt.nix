{ inputs, ... }: {
  imports = [ inputs.treefmt-nix.flakeModule ];

  perSystem = _: {
    # https://flake.parts/options/treefmt-nix.html
    treefmt = {
      projectRootFile = "flake.nix";

      programs = {
        nixfmt = {
          enable = true;
          priority = 1;
        };
        deadnix = {
          enable = true;
          priority = 2;
        };
        statix = {
          enable = true;
          priority = 3;
        };
      };
    };
  };
}

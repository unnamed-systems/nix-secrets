{ lib, ... }: {
  debug = true;

  systems = lib.systems.flakeExposed;

  imports = [
    ./parts/shell.nix
    ./parts/treefmt.nix
    ./tests/default.nix
  ];
}

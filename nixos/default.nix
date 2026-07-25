{ lib, ... }:
{
  imports = [
    ./activate
    ./options
    ./ciMode.nix
    ./env.nix
  ];
}

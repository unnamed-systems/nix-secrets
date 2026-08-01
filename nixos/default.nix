{ lib, ... }:
{
  imports = [
    ./activate
    ./options
    ./ciMode.nix
    ./defaultRecipients.nix
    ./env.nix
  ];
}

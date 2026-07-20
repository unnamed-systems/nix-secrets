{
  description = "TODO";

  inputs = {
    nixpkgs.url = "https://nixos.org/channels/nixos-unstable/nixexprs.tar.xz";

    nix-secrets.url = "github:unnamed-systems/nix-secrets";
  };

  outputs = { nixpkgs, nix-secrets, ... }: {
    nixosConfigurations = {
      "test" = nixpkgs.lib.nixosSystem {
        modules = [
          ./configuration.nix
          nix-secrets.nixosModules.default
        ];
      };
    };
  };
}

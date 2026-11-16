{ inputs, ... }: {
  perSystem = { pkgs, ... }: {
    apps.serve-docs = {
      type = "app";
      program = pkgs.writeShellApplication {
        name = "serve-docs";
        runtimeInputs = [ pkgs.serve ];
        text = ''
          exec serve --symlinks ${inputs.nix-secrets.packages.x86_64-linux.docs} "$@"
        '';
      };
    };
  };
}

{
  pkgs,
  nixosModule,
  shared,
  ...
}:
pkgs.testers.runNixOSTest (_: {
  name = "rekey";

  nodes.machine = {
    imports = [
      nixosModule
      shared.defaultNoActivate
    ];

    security.nix-secrets.secrets = {
      password.recipients = [ "age124q3hcmalxqert4uml2tfqg43eeasvpfnl6wzjh0ldupkfd99uhsvapvp7" ];
    };
  };

  testScript =
    { nodes, ... }:
    ''
      machine.succeed("cp -r ${nodes.machine.security.nix-secrets.storage} /tmp/rekey-storage")

      hash_before = machine.succeed("sha256sum /tmp/rekey-storage/password.enc").strip().split()[0]
      hash_for_users_before = machine.succeed("sha256sum /tmp/rekey-storage/passwordForUsers.enc").strip().split()[0]

      pre_password = machine.succeed(
        "nix-secrets -f ${shared}#shared decrypt password --storage /tmp/rekey-storage --output /dev/stdout"
      ).strip()
      pre_for_users = machine.succeed(
        "nix-secrets -f ${shared}#shared decrypt passwordForUsers --storage /tmp/rekey-storage --output /dev/stdout"
      ).strip()

      machine.succeed(
        "nix-secrets -f ${shared}#shared rekey --storage /tmp/rekey-storage"
      )

      hash_after = machine.succeed("sha256sum /tmp/rekey-storage/password.enc").strip().split()[0]
      hash_for_users_after = machine.succeed("sha256sum /tmp/rekey-storage/passwordForUsers.enc").strip().split()[0]

      assert hash_before != hash_after, "password.enc did not change after rekey"
      assert hash_for_users_before != hash_for_users_after, "passwordForUsers.enc did not change after rekey"

      post_password = machine.succeed(
        "nix-secrets -f ${shared}#shared decrypt password --storage /tmp/rekey-storage --output /dev/stdout"
      ).strip()
      post_for_users = machine.succeed(
        "nix-secrets -f ${shared}#shared decrypt passwordForUsers --storage /tmp/rekey-storage --output /dev/stdout"
      ).strip()

      assert pre_password == post_password, "password value changed after rekey"
      assert pre_for_users == post_for_users, "passwordForUsers value changed after rekey"
    '';
})

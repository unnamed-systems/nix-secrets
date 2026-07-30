# Configuring nix-secrets

An in-depth guide to configuring `nix-secrets`.

## Misc options

Two options you should know about are `security.nix-secrets.storagePath` and `security.nix-secrets.nixEvalCommand`.

### storagePath

The first one, `storagePath`, allows you to define an absolute path to your storage directory which you usually pass to the `--storage` cli flag. If defined, the cli will no longer require you to pass the `--storage` flag.

Example:

```nix
{
    security.nix-secrets = {
        storagePath = "/home/user/nixos-config/secrets";
    };
}
```

### nixEvalCommand

First, you need to understand how the cli works. To get secret information when you initiate the `edit`, `rekey` or `regenerate` commands, the cli evaluates your config by appending `.#nixosConfigurations.<hostname>.security.nix-secrets.manifest` to `nixEvalCommand`.

So, as you may've guessed, if you use a non-standard installation of Nix, be it changing the command or flags, you will have to adapt `nixEvalCommand` to your use case. The default value is `${lib.getExe config.nix.package} --extra-experimental-features 'nix-command flakes' eval --raw`.

**Do note that this is an advanced configuration option and you should generally be fine with the default value (tested on Nix and Lix).**

## Secrets

To define secrets, as you may've learned from the README, you have to set the `security.nix-secrets.secrets.<name>` option. 

The `name` option is important, as `nix-secrets` is the one managing your secret placement, unlike in `sops-nix` or `agenix`. Think of the name as the path in the storage directory too — if you have a secret named `cloudflare/dnsToken` it will end up in `./cloudflare/dnsToken.enc` inside your storage directory.

Refer to the [section above](#recipients) regarding `recipients`.

Now, let's talk about options you will use most often first.

### Permissions

You can set `owner`, `group` and `mode` for every secret. The first two can take either a string with user/group name or a number representing the uid/gid. `mode`, on the other hand, is a string representing the octal permissions. You can omit the leading zero if you wish.

For example:

```nix
{
  security.nix-secrets = {
    secrets = {
      sshkey = { # Will result in a secret owned by `user` (no group!) with the `-rw-------` permissions.
        mode = "0600";
        owner = "user";
      };
      "services/navidrome/apiKey" = { # Will result in a secret owned by a user with the uid of `1000` and the group `media` with the `-r--------` permissions.
        mode = "400";
        owner = 1000;
        group = "media";
      };
    };
  };
}
```

Do note that you can **not** set the `owner` or `group` for secrets marked as `neededForUsers`.


### Recipients

Every secret has a `recipients` option. This option can either take raw recipients or aliases from `security.nix-secrets.recipientAliases`. Every alias can take multiple recipients. To demonstrate it better, consider the following:

```nix
{
  security.nix-secrets = {
    recipientAliases = {
      pc = "age14e2jdmau7tpau9emcn6gmg26vfl0uyf6cfd9lz85jml6ttv9wq2qphps4t"; # Define your recipients!
      laptop = = "age12jegn2g3awkzfja7egt9mu0ld9rv78c7rlpcgs9vn9gf0ndrlgfq2gn5rk";
      servers = ["age1ls5m8ml8cdu202xakl56lspqrccgln4kfx8q7c6v7qdex92xryhs03v6re" "age10n7r8daletpkqjupy3r6vqvnkq8a4dzq3w8lkjjsdz6754ku74zqdkxjc4"]; # Lists also work
    };

    secrets = {
      password.recipients = [ "pc" "laptop" "age1m3z7s07lykg0khllv65yxuapyeyul33c2pl807f3qlpg0aey6gqss6v6pe" ]; # Or define recipients per-secret
      sshKey.recipients = [ "servers" "laptop" "pc" ];
    };
  };
}
```

### Path

To actually use the secret you need its path. You would usually pass it to a NixOS option, say, `services.vaultwarden.environmentFile`. 

This is where you'd use the `path` option, which is a path to the resulting decrypted secret file. For example:

```nix
{
    services.vaultwarden = {
        environmentFile = config.security.nix-secrets.secrets."vaultwarden/env".path;
    };
}
```

However, this is not all. You can also set the `path` option when creating the secret. If you do so, `nix-secrets` will symlink your file to your desired location. 


This can be useful when dealing with programs that don't allow you to pass a file for secrets, instead expecting one on disk in a particular location.

For example:
```nix
{
  security.nix-secrets = {
    secrets = {
      "vaultwarden/env".path = "/home/user/.config/vaultwarden/env"; # Made-up path, but you get the point.
    };
  };
}
```

### neededForUsers

The activation script runs after NixOS creates the users. This means that it's not possible to set `users.users.<name>.hashedPasswordFile` to a regular secret managed by `nix-secrets`.

To actually do that, you should set the `neededForUsers` secret option. This makes the secret be decrypted to `/run/nix-secrets-for-users` before user creation. Due to this, it's impossible to set `owner` and `group` options for such secrets.

For example:
```nix
{
    security.nix-secrets.secrets.password.neededForUsers = true;
    # The secret should contain the hashed user password
    # you can generate one by using `echo "password" | mkpasswd -s`
    # for example: `$y$j9T$uYFYCkJ20.oc4oIzPHzDc0$lpBKkRUxBOn3E9zR8Dmj6to3z0JqXTLQwQSHtE86/b2`

    users.users.user = {
        isNormalUser = true;
        hashedPasswordFile = config.security.nix-secrets.secrets.password.path;
    };
}
```

If you are using impermanence, make sure your age identities are loaded early enough. You have two choices.

1. Make sure the identity is on a filesystem loaded before the activation script.
```nix
security.nix-secrets.identityPaths = [ "/persist/home/user/keys.txt" ];
```

2. Persist the filesystem the key is on.
```nix
fileSystems."/home/user".neededForBoot = true;
```
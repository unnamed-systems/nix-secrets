# Configuring nix-secrets

An in-depth guide to configuring `nix-secrets`.

## Misc options

Two options you should know about are `security.nix-secrets.storagePath` and `security.nix-secrets.nixEvalCommand`.

### storagePath

The first one, `storagePath`, allows you to define an absolute path to your storage directory which you usually pass to the `--storage` CLI flag. If defined, the CLI will no longer require you to pass the `--storage` flag.

Example:

```nix
{
  security.nix-secrets = {
    storagePath = "/home/user/nixos-config/secrets";
  };
}
```

You can also set the `NIX_SECRETS_STORAGE_PATH` variable for a similar effect, e.g. in per-developer environments. The order of precedence is `--storage argument` > `environment variable` > `storagePath option`.

### nixEvalCommand

First, you need to understand how the CLI works. To get secret information when you initiate the `edit`, `rekey` or `regenerate` commands, the CLI evaluates your config by appending `.#nixosConfigurations.<hostname>.security.nix-secrets.manifest` to `nixEvalCommand`.

So, as you may've guessed, if you use a non-standard installation of Nix, be it changing the command or flags, you will have to adapt `nixEvalCommand` to your use case. The default value is `${lib.getExe config.nix.package} --extra-experimental-features 'nix-command flakes' eval --raw`.

**Do note that this is an advanced configuration option and you should generally be fine with the default value (tested on Nix and Lix).**

## Secrets

To enable nix-secrets, set `security.nix-secrets.enable` to `true`.

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

Every secret has a `recipients` option. This option accepts either raw recipients or aliases defined in `security.nix-secrets.recipientAliases`. In addition, `security.nix-secrets.defaultRecipients` lets you specify recipients that are automatically added to every secret. To disable the default recipients for a specific secret, set `recipients = lib.mkForce [ ... ]`. Every alias can take multiple recipients. The following example demonstrates how this works:

```nix
{ lib, ... }: {
  security.nix-secrets = {
    recipientAliases = {
      pc = "age14e2jdmau7tpau9emcn6gmg26vfl0uyf6cfd9lz85jml6ttv9wq2qphps4t"; # Define your recipients!
      laptop = "age12jegn2g3awkzfja7egt9mu0ld9rv78c7rlpcgs9vn9gf0ndrlgfq2gn5rk";
      master = "age1gkq97hxujlxs2k4zwhnqxdadd6mds46m7nudgw5l02egldkl44ds4h5vqy";
      servers = ["age1ls5m8ml8cdu202xakl56lspqrccgln4kfx8q7c6v7qdex92xryhs03v6re" "age10n7r8daletpkqjupy3r6vqvnkq8a4dzq3w8lkjjsdz6754ku74zqdkxjc4"]; # Lists also work
    };

    defaultRecipients = ["master" "age1xrpq2tjpsp564c4rnmq7yflvc6ds2e4j48f49z4tm9j8wqa2nf4sm8nch4"];

    secrets = {
      password.recipients = [ "pc" "laptop" "age1m3z7s07lykg0khllv65yxuapyeyul33c2pl807f3qlpg0aey6gqss6v6pe" ]; # Or define recipients per-secret
      sshKey.recipients = [ "servers" "laptop" "pc" ];
      pcOnly.recipients = lib.mkForce [ "pc" ] # Prevent default recipients from being added
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
{
  security.nix-secrets.identityPaths = [ "/persist/home/user/keys.txt" ];
}
```

2. Persist the filesystem the key is on.
```nix
{
  fileSystems."/home/user".neededForBoot = true;
}
```

## Templates

Templates are designed for creating files with secrets inside. To use them, define `security.nix-secrets.templates`:

```nix
{
  security.nix-secrets = {
    templates = {
      forgejoEnv.content = ''
        NAME="Git Server"
        TURNSTILE_SECRET="''${config.security.nix-secrets.secrets."forgejo/turnstile/secret".templateKey}"
        TURNSTILE_SITEKEY="''${config.security.nix-secrets.secrets."forgejo/turnstile/sitekey"}" // Shorthand for `templateKey`
      '';
    };
  };
}
```

To embed secrets, interpolate either `config.security.nix-secrets.secrets.<name>.templateKey` or, as a shorthand, `config.security.nix-secrets.secrets.<name>`, which will be coerced to a string.

If you can't interpolate values in your template content, calculate them yourself, as they are just hashes based on the secret name: `{{NIX_SECRETS-${builtins.hashString "sha256" config.name}}}`.

Templates support the same options as secrets, namely `owner`, `group`, `mode`, `path` and `neededForUsers`.

## Generators

Secrets can support generators to bootstrap secret values or derive secrets based on other secrets. 

Each generator is a function that accepts an attribute set of arguments and returns a package or a generator specification.

A generator without arguments can be referenced by name:

```nix
{
  security.nix-secrets.secrets.my-secret = {
    generator = "uuid";
  };
}
```

A generator with arguments can be referenced using an attribute set:

```nix
{
  security.nix-secrets.secrets.my-secret = {
    generator.uuid = {
      count = 5;
      raw = true;
    };
  };
}
```

To define custom generators, set the `generator` argument to a package:

```nix
{
  security.nix-secrets.secrets.my-secret = {
    generator = pkgs.writeShellScript "password-generator" ''
      tr -dc 'A-Za-z0-9' < /dev/urandom | head -c 16; echo "-pa$$w0rd"
    '';
  }
}
```

For a list of default generators, consult [the reference](https://nix-secrets.unnamed.systems/generators_reference)

### Custom generators

You can also create custom generators to reference by name like those included by default. To do so, define a new generator in `security.nix-secrets.generators`:

```nix
{
  config.security.nix-secrets.generators.custom-uuid =
    {
      count ? 1,
      raw ? false,
    }:
    pkgs.writeShellScript "uuid-generator" ''
      uuidgen ${
        lib.cli.toCommandLineShellGNU { } {
          random = true;
          inherit count;
        }
      }${lib.optionalString raw " | tr -d '-'"}
    '';
}
```

Your generators can also reference existing ones:

```nix
{
  security.nix-secrets.generators.custom-ssh-ed25519 =
    {
      format ? null,
    }:
    {
      ssh = { # References the default `ssh` generator
        type = "ed25519";
        inherit format;
      };
    };
}
```

# A note for using post-quantum secrets

Due to the [rage](https://github.com/str4d/rage) library not supporting post-quantum encryption natively yet (as of writing, `nix-secrets` uses rage v0.12.1), you have to use the `age` package and the `age-plugin-pq` plugin.

Add the `age` package to `extraPackages`:

```nix
{ pkgs, ... }: {
  security.nix-secrets = {
    extraPackages = with pkgs; [age];
  };
}
```

Then, convert your secrets to the `age-plugin-pq` format. This is required until `rage` supports post-quantum keys. The public keys remains unchanged.

```bash
echo "AGE-SECRET-KEY-PQ-10FQNUZZCAG78VJM4H5DGHNGYM3YZTN3YFG2YEJG8LUVTDW9XASAQJXUA86" | age-plugin-pq -identity
```

Sorry for the inconvenience.

Also note that you can **not** mix post-quantum recipients with regular ones, per age design.
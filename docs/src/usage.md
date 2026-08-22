# CLI usage

A guide to the CLI commands you'll use most often.

You can access the information by using the `nix-secrets help` command, or just running the binary without any arguments.

All commands take the `--flake` argument, which is in format of a `nixos-rebuild` flake, for example, `.#pc`. This argument is used to evaluate your `nix-secrets` nix configuration and defaults to `.`, which means it will use your current working directory along with your system hostname.

## Editing secrets

To edit a secret, navigate to your flake directory and use the `nix-secrets edit` command. It has the following structure:

```bash
nix-secrets edit --storage <STORAGE> <NAME>
```

The `--storage` argument is used to specify your secret storage directory, defined by `security.nix-secrets.storage`. You can set the [storagePath](./CONFIGURE.md#storagepath) configuration option to automatically provide the default value or use the `NIX_SECRETS_STORAGE_PATH` environment variable.

The `name` option is the name of the secret, not the path to its file. For example, if you have the `cloudflare/dnsToken` secret, you would pass `cloudflare/dnsToken` in the CLI **and not** `./secrets/cloudflare/dnsToken.enc`.

## Rekeying

After you edit your secret `recipients` option, you have to actually apply the changes to the secret file. This is called rekeying.

To rekey a particular set of secrets, use the `rekey` command.

```bash
nix-secrets rekey --storage <STORAGE> [SECRETS]
```

The `--storage` argument is the same as with the [edit](#editing-secrets) command.

The `SECRETS` argument is a list of secret names you want to rekey separated by spaces, for example, `cloudflare/dnsToken password`. If omitted, will rekey all secrets in your config.
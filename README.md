# Nix Secrets

![Created by a human with a heart](./.assets/images/human.png)
![We do not accept LLM assisted contributions](./.assets/images/no-ai.png)
![Written in Rust](./.assets/images/rust.png)
![Powered by Nix](./.assets/images/nixos.png)
![Licensed under GPL 3.0](./.assets/images/gpl.png)

---

`nix-secrets` is a NixOS secret management solution designed to be lightweight and simple to use. It comes as a NixOS module which automatically installs the `nix-secrets` binary used for secret management. The secrets themselves are encrypted using the [age](https://age-encryption.org/v1) encryption format.

## Advantages

The main advantages over existing solutions like `sops-nix` and `agenix` are:

- Entirely managed in the nix module system. No need for `.sops.yaml` or standalone `secrets.nix`.
- Supports templates, placeholders and generators.
- Aims to be as atomic as possible with no intermediate state. No imperative bash scripts.
- Utilizes built-in age asymmetrical encryption, resulting in no public key leakage in encrypted files unlike `sops-nix` or `agenix`. This means you can have your secrets managed in a private repository without ever showing your public keys.
- Small binary and closure size. The `nix-secrets` binary is less than 2 Mb, compared to `sops` being over 50 Mb, for example.

## Quick Installation

Add the input to your `flake.nix`:

```nix
inputs.nix-secrets.url = "github:unnamed-systems/nix-secrets";
```

Then add the module to your system modules:

```nix
imports = [ inputs.nix-secrets.nixosModules.default ];
```

The module automatically adds the `nix-secrets` binary to your `environment.systemPackages`.

To generate a key you can use the provided `nix-secrets` CLI or any other age compatible one (`age-keygen`, `rage-keygen`).
```bash
nix-secrets keygen [-o <output>]
```

For further CLI usage consult [the documentation](./docs/USAGE.md) or the manpages.

To enable the module itself, create a `secrets` directory and enable the following options:

```nix
{
  security.nix-secrets = {
    storage = ../secrets; # Relative path to your `secrets` (copied to /nix/store)
    identityPaths = [ "/home/user/keys.txt" ]; # Path to your age private key
    recipientAliases = {
      first = "age14e2jdmau7tpau9emcn6gmg26vfl0uyf6cfd9lz85jml6ttv9wq2qphps4t"; # Your age recipient (public key)
    };

    secrets = {
      password.recipients = [ "first" ]; # Add your secrets here...
    };
  };
}
```

Before actually adding the secret to your storage directory, define it in the config as previously shown. Afterwards, make sure to add your `secrets` directory to git, rebuild your system and run the command to edit your secret:

```bash
nix-secrets edit password --storage secrets
```

`password` being your secret name you've defined previously and `secrets` is the relative directory to your storage. You can set the `security.nix-secrets.storagePath` to an absolute path (e.g. `/home/user/nixos-config/secrets`) to eliminate the need to pass the `storage` argument.

After editing the secret, add the resulting file to git and rebuild your system again. The secret should now be available under it's default path in `/run/nix-secrets`.

For advanced configuration refer to [the documentation](./docs/CONFIGURE.md)
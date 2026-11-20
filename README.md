<p>
  <picture>
    <source media="(prefers-color-scheme: light)" srcset="./.github/assets/logo_white.svg">
    <source media="(prefers-color-scheme: dark)" srcset="./.github/assets/logo_dark.svg">
    <img src="./.github/assets/logo_white.svg" width="300" alt="nix-secrets logo">
  </picture>
  <img align="right" src="./.github/assets/badges/nixos.png" alt="Powered by Nix">
  <img align="right" src="./.github/assets/badges/rust.png" alt="Written in Rust">
  <img align="right" src="./.github/assets/badges/no-ai.png" alt="We do not accept LLM-assisted contributions">
  <img align="right" src="./.github/assets/badges/human.png" alt="Created by a human with a heart">
</p>

`nix-secrets` is a lightweight and simple to use secret management solution for NixOS, Home Manager, nix-darwin, and Hjem. It provides a module that automatically installs the `nix-secrets` CLI, allowing you to manage encrypted secrets directly through the Nix module system.

Secrets are encrypted using [age](https://age-encryption.org/v1), a modern and simple encryption format.

## Advantages

Compared to existing solutions such as `sops-nix` and `agenix`, `nix-secrets` offers several advantages:

- Entirely managed in the nix module system. No need for `.sops.yaml` or standalone `secrets.nix`.
- Supports templates, placeholders and generators.
- Aims to be as atomic as possible with no intermediate state. No imperative bash scripts.
- Utilizes built-in age asymmetrical encryption, resulting in no public key leakage in encrypted files unlike `sops-nix` or `agenix`. This means you can have your secrets managed in a private repository without ever showing your public keys.
- Small binary and closure size. The `nix-secrets` binary is less than 2 Mb, compared to `sops` being over 50 Mb, for example.

## Quick Installation

Add `nix-secrets` to your flake inputs:

```nix
inputs.nix-secrets.url = "github:unnamed-systems/nix-secrets";
# Mirror "git+https://codeberg.org/unnamed-systems/nix-secrets"
```

Import the NixOS module in your system configuration:

```nix
imports = [
  inputs.nix-secrets.nixosModules.default
];
```

The module automatically adds the `nix-secrets` CLI to `environment.systemPackages`.

### Key Generation

Generate a new age identity using the provided CLI:

```bash
nix-secrets keygen [-o <output>]
```

Alternatively, you can use any age-compatible key generation tool, such as `age-keygen` or `rage-keygen`.

For more information about CLI commands, see the [CLI documentation](https://nix-secrets.unnamed.systems/usage) or consult the man pages.

### Configuration

To enable secret management, create a `secrets` directory in your configuration repository and configure the module:

```nix
{ lib, ... }: {
  security.nix-secrets = {
    enable = true;

    # Path to your secrets directory (copied to the Nix store).
    # For improved security, it is recommended to keep the storage in a private repository and reference it through a flake input.
    storage = ../secrets;

    # Paths to your age identities.
    # For NixOS it defaults to the "/var/lib/nix-secrets/identities.txt" and the RSA and ED25519 SSH keys from the `services.openssh.hostKeys`.
    # For Home Manager and Hjem it defaults to the "~/nix-secrets/identities.txt".
    # For nix-darwin it defaults to the "/Library/Application Support/nix-secrets/identities.txt" and the RSA and ED25519 SSH keys from the `services.openssh.hostKeys`.
    identityPaths = [
      "/home/user/keys.txt"
      "/home/user/.ssh/id_ed25519" # SSH keys are also supported.
    ];

    # Define recipient aliases for easier secret configuration.
    recipientAliases = {
      master = "age14e2jdmau7tpau9emcn6gmg26vfl0uyf6cfd9lz85jml6ttv9wq2qphps4t";
      ssh = "ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAIEuUsB0HH//1qkvgQMWTEoNd0riZpk+8A5w1Ep2vGKk0";
      all = [ "master" "ssh" ];
    };

    # Recipients automatically added to every secret.
    # To override this list for a specific secret, use `lib.mkForce`.
    defaultRecipients = [ "master" ];

    # Declare your secrets and their recipients.
    secrets = {
      password = {
        recipients = [ "all" ];
        # Whether the secret must be available before users and groups are created.
        neededForUsers = true;
      };
      signingKey = {
        recipients = lib.mkForce [ "ssh" ];
        generator = "ssh-ed25519";
      };
      something = {
        recipients = [ "age1j0qx6wdh4ncr3dvexuy9ysu5mrpkp5p7rldt8j7mc6r4xkzngcpqeuk403" ];
        path = "/var/lib/something";
        mode = "0440";
        owner = "alice";
        group = "users";
      };
    };
  };
}
```

After declaring a secret in your configuration, create or edit it using:

```bash
nix-secrets edit <secret-name> --storage ./secrets
```

 You can set the `security.nix-secrets.storagePath` to an absolute path (e.g. `/etc/nixos/configurations/secrets`) to eliminate the need to pass the `storage` argument.

### Further Configuration

For advanced configuration options, templates, placeholders, and generators, refer to the [configuration documentation](https://nix-secrets.unnamed.systems/configure).

## Reference

For a complete list of available options, see the following references:

- [NixOS options](https://nix-secrets.unnamed.systems/nixos-configuration-options)
- [Home Manager options](https://nix-secrets.unnamed.systems/home-manager-configuration-options)
- [Hjem options](https://nix-secrets.unnamed.systems/hjem-configuration-options)
- [nix-darwin options](https://nix-secrets.unnamed.systems/nix-darwin-configuration-options)

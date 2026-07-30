# Contributing Guidelines

You are welcome to contribute to the project. Don't be afraid to ask if you're
unsure about anything, we will kindly help you.

**We do not accept LLM assisted contributions. No generative AI technology has been used in the process of creating this software at any stage. Please do not try to contribute LLM-assisted code and waste our time.**

## Code Style

The code is formatted by `nixfmt-tree` on the nix side and `cargo fmt` on the rust side. There is a devshell available which you can activate by either using `nix develop` or, if you use direnv, `direnv allow`.

Before committing, please run automated tests using `nix flake check .#`. You may also test the rust part by running `cargo test`, which is included in the former.

## Commit Style

All commits are to follow the [seven rules](https://cbea.ms/git-commit/), **except** capitalizing the subject lines.
Our commit template is:

```
{scope}: {description}

{body}
```

Let's make an example commit.

The `scope` is what this commit changes, for example, `src/manifest.rs`.

Say, we added templates support to the manifest. The commit would look like this:

```
src/manifest: add templates support

Add the `templates` struct to achieve feature parity with the nix side 
```

If your commit is breaking you should add an exclamation mark after scope.

```
nixos/activate!: rename `extraPackages` to `plugins`
```

All commits in PRs are **encouraged** to be atomic, though not strictly
required.

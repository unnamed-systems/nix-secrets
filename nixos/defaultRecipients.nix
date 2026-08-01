{
  lib,
  config,
  ...
}:
let
  cfg = config.security.nix-secrets;
in
{
  options.security.nix-secrets.defaultRecipients = lib.mkOption {
    description = "TODO";
    type = lib.types.listOf lib.types.str;
    default = [ ];
    apply =
      values:
      lib.uniqueStrings (builtins.concatMap (value: cfg.recipientAliases.${value} or [ value ]) values);
    example = lib.literalExpression ''[ config.networking.hostname "someAlias" "age1nr6qkv2y49g5pvkswskyy3nzazsp9wy3gxyxzxlutqgq2ejec4xqpmjzh5" ]'';
  };
}

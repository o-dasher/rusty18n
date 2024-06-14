{ ... }:
{
  languages.rust = {
    enable = true;
    channel = "nightly";
  };

  pre-commit.hooks = {
    rustfmt.enable = true;
    clippy.enable = true;
  };
}

{ ... }:
{
  languages.rust = {
    enable = true;
    channel = "nightly";
    components = [
      "rustc"
      "cargo"
      "clippy"
      "rustfmt"
      "rust-analyzer"
    ];
  };

  pre-commit.hooks = {
    rustfmt.enable = true;
    clippy.enable = true;
  };
}

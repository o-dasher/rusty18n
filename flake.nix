{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";

    flake-parts = {
      url = "github:hercules-ci/flake-parts";
      inputs.nixpkgs-lib.follows = "nixpkgs";
    };

    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs =
    { fenix, flake-parts, ... }@inputs:
    flake-parts.lib.mkFlake { inherit inputs; } {
      systems = [
        "x86_64-linux"
        "aarch64-linux"
      ];

      perSystem =
        { pkgs, system, ... }:
        let
          toolchain = fenix.packages.${system}.stable;
        in
        {
          devShells.default = pkgs.mkShell {
            packages =
              (with pkgs; [ nixfmt-rfc-style ])
              ++ (with toolchain; [
                clippy
                rustfmt
                rust-analyzer
                rust-src
                rustc
                cargo
              ]);
          };
        };
    };
}

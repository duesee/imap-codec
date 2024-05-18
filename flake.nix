{
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-23.11";

    # The rustup equivalent for Nix.
    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
    };

    # Allows non-flakes users to still be able to `nix-shell` based on
    # `shell.nix` instead of this `flake.nix`.
    flake-compat = {
      url = "github:edolstra/flake-compat";
      flake = false;
    };
  };

  outputs = { self, nixpkgs, fenix, ... }:
    let
      inherit (nixpkgs) lib;

      eachSupportedSystem = lib.genAttrs supportedSystems;
      supportedSystems = [
        "x86_64-linux"
        "aarch64-linux"
        "x86_64-darwin"
        "aarch64-darwin"
      ];

      mkDevShells = system:
        let
          pkgs = import nixpkgs { inherit system; };

          # get the rust toolchain from the rustup
          # `rust-toolchain.toml` configuration file
          rust-toolchain = fenix.packages.${system}.fromToolchainFile {
            file = ./rust-toolchain.toml;
            sha256 = "opUgs6ckUQCyDxcB9Wy51pqhd0MPGHUVbwRKKPGiwZU=";
          };

        in
        {
          default = pkgs.mkShell {
            buildInputs = [ rust-toolchain ];
          };
        };

    in
    {
      devShells = eachSupportedSystem mkDevShells;
    };
}

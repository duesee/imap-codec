{
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-25.05-small";
  };

  outputs =
    { self, nixpkgs, ... }:
    let
      eachSupportedSystem = nixpkgs.lib.genAttrs supportedSystems;
      supportedSystems = [
        "x86_64-linux"
        "aarch64-linux"
        "x86_64-darwin"
        "aarch64-darwin"
      ];

      mkDevShells =
        system:
        let
          pkgs = import nixpkgs { inherit system; };
        in
        {
          default = pkgs.mkShell {
            strictDeps = true;
            nativeBuildInputs = with pkgs; [
              just
              rustPlatform.bindgenHook
              rustup
            ];
          };
        };
    in
    {
      devShells = eachSupportedSystem mkDevShells;
    };
}

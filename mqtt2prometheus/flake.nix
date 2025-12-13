  {
    description = "Build mqtt2prometheus with Go via flake";

    inputs.nixpkgs.url = "github:NixOS/nixpkgs/nixos-24.05";

    outputs = { self, nixpkgs }:
      let
        forAllSystems = nixpkgs.lib.genAttrs [ "x86_64-linux" "aarch64-linux" ];
      in {
        devShells = forAllSystems (system:
          let pkgs = import nixpkgs { inherit system; };
          in {
            default = pkgs.mkShell {
              packages = with pkgs; [ go git gnumake ];
            };
          });
      };
  }

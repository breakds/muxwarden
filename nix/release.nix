{ inputs, ... }:

let
  inherit (inputs) self nixpkgs;
  
in {
  flake.overlays.default = nixpkgs.lib.composeManyExtensions [
    (final: prev: {
      muxwarden = final.callPackage ./pkgs/muxwarden {};
    })
  ];
  
  perSystem = { system, pkgs, lib, ... }: {
    _module.args.pkgs = import nixpkgs {
      inherit system;
      config = {
        allowUnfree = true;
      };
      overlays = [ self.overlays.default ];
    };

    packages.default = pkgs.muxwarden;
  };
}

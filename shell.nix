{ pkgs ? import <nixpkgs> {} }:
 {
  qpidEnv = pkgs.stdenvNoCC.mkDerivation {
    name = "dev-shell";
    nativeBuildInputs = with pkgs.buildPackages; [
      gcc13
      pkg-config
    ];
  };
}

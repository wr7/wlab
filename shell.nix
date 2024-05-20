{ pkgs ? import <nixpkgs> {} }:

pkgs.mkShell  {
  name = "dev-shell";
  nativeBuildInputs = with pkgs.buildPackages; [
    libffi
    libxml2
    pkg-config
    llvmPackages_17.libllvm
  ];
}

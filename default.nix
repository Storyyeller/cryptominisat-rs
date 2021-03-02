let
  pkgs = import <nixpkgs> {};
  cryptominisat = pkgs.stdenv.mkDerivation {
    name = "cryptominisat";
    src = ./cryptominisat;
    nativeBuildInputs = with pkgs; [
      cmake
      pkg-config
    ];
    buildInputs = with pkgs; [
      gmp
      zlib
      boost
      python3
      m4ri
    ];
  };
in
pkgs.stdenv.mkDerivation {
  name = "cryptominisat-rs";
  src = builtins.fetchGit ./.;
  buildInputs = with pkgs; [
    cryptominisat
  ];
  propagatedBuildInputs = with pkgs; [
    rustup
    cargo-make
    cargo
  ];
}


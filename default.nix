{ pkgs ? import <nixpkgs> {} }:
let
    sources = import ./nix/sources.nix;
    nixpkgs-mozilla = import sources.nixpkgs-mozilla pkgs pkgs;
    rustChannel = nixpkgs-mozilla.rustChannelOf {
        date = "2020-11-12";
        channel = "nightly";
    };
    naersk = pkgs.callPackage sources.naersk rec {
        rustc = rustChannel.rust;
        cargo = rustc;
    };
    inherit (import sources.gitignore { inherit (pkgs) lib; }) gitignoreSource;
in naersk.buildPackage {
    src = gitignoreSource ./.;
    buildInputs = with pkgs; [ openssl pkgconfig ];
    copyBinsFilter = ''select(.reason == "compiler-artifact" and .executable != null and .profile.test == false and .target.name == "upd8r")'';
}

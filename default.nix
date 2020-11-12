let
    sources = import ./nix/sources.nix;
    nixpkgs-mozilla = import sources.nixpkgs-mozilla;
    pkgs = import <nixpkgs> {
        overlays = [
            nixpkgs-mozilla
            (self: super: rec {
                rustChannel = self.rustChannelOf {
                    date = "2020-11-12";
                    channel = "nightly";
                };
                rustc = rustChannel.rust;
                cargo = rustc;
            })
        ];
    };
    naersk = pkgs.callPackage sources.naersk {};
    inherit (import sources.gitignore { inherit (pkgs) lib; }) gitignoreSource;
in naersk.buildPackage {
    src = gitignoreSource ./.;
    buildInputs = with pkgs; [ openssl pkgconfig ];
    copyBinsFilter = ''select(.reason == "compiler-artifact" and .executable != null and .profile.test == false and .target.name == "upd8r")'';
}
